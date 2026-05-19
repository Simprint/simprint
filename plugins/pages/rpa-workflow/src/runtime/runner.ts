import type { RpaCapturedSelector, RpaWorkflowSchema, RpaWorkflowStep } from '../types';
import type { BrowserAdapter, BrowserAdapterRunResult, BrowserSession } from './browser-adapter';

export type RpaRunnerStepStatus = 'pending' | 'running' | 'awaiting_input' | 'success' | 'failed';

export interface RpaRunnerStepEvent {
  stepId: string;
  stepType: string;
  status: RpaRunnerStepStatus;
  error?: string;
  loopIteration?: number;
  loopTotal?: number;
}

export interface RpaRunnerCapturedTargetEvent {
  stepId: string;
  stepType: string;
  captured: RpaCapturedSelector;
}

export interface RpaRunnerScreenshotEvent {
  stepId: string;
  stepType: string;
  screenshot: string;
}

export interface RpaRunnerExtractedDataEvent {
  stepId: string;
  stepType: string;
  value: string;
}

export interface RpaRunnerOptions {
  onStepStatusChange?: (event: RpaRunnerStepEvent) => void;
  onTargetCaptured?: (event: RpaRunnerCapturedTargetEvent) => void;
  onScreenshotCaptured?: (event: RpaRunnerScreenshotEvent) => void;
  onDataExtracted?: (event: RpaRunnerExtractedDataEvent) => void;
}

type StepExecutionResult =
  | {
      type: 'next';
      nextStepId: string | null;
    }
  | {
      type: 'break_loop';
    };

export class RpaRunner {
  constructor(private readonly adapter: BrowserAdapter) {}

  async run(
    workflow: RpaWorkflowSchema,
    session: BrowserSession,
    options?: RpaRunnerOptions
  ): Promise<BrowserAdapterRunResult> {
    if (!workflow.start_step_id) {
      throw new Error('Workflow has no start step');
    }

    const stepMap = new Map(workflow.steps.map((step) => [step.id, step]));
    const screenshots: string[] = [];
    const variables = this.initializeWorkflowVariables(workflow);
    let currentStepId: string | null = workflow.start_step_id;

    await this.adapter.connect(session);
    try {
      while (currentStepId) {
        const step = stepMap.get(currentStepId);
        if (!step) {
          throw new Error(`Unknown workflow step: ${currentStepId}`);
        }

        const result = await this.runStep(step, stepMap, screenshots, variables, options);
        if (result.type === 'break_loop') {
          throw new Error('BREAK_LOOP_OUTSIDE_LOOP');
        }

        currentStepId = result.nextStepId;
      }
    } finally {
      this.adapter.close();
    }

    return { screenshots };
  }

  private async runStep(
    step: RpaWorkflowStep,
    stepMap: Map<string, RpaWorkflowStep>,
    screenshots: string[],
    variables: Record<string, unknown>,
    options?: RpaRunnerOptions
  ): Promise<StepExecutionResult> {
    options?.onStepStatusChange?.({
      stepId: step.id,
      stepType: step.type,
      status: 'running',
    });

    try {
      let nextStepId: string | null;

      switch (step.type) {
        case 'goto':
          if (!step.url) {
            throw new Error(`Step ${step.id} is missing url`);
          }
          await this.adapter.goto(this.resolveTemplate(step.url, variables), { waitUntil: step.wait_until });
          nextStepId = step.next;
          break;

        case 'select_tab': {
          const tabIndex =
            typeof step.config?.tabIndex === 'number' && Number.isFinite(step.config.tabIndex)
              ? Math.trunc(step.config.tabIndex)
              : NaN;

          if (!Number.isInteger(tabIndex) || tabIndex < 1) {
            throw new Error('TAB_INDEX_INVALID');
          }

          await this.adapter.selectTab(tabIndex);
          nextStepId = step.next;
          break;
        }

        case 'close_tab': {
          const tabIndex =
            typeof step.config?.tabIndex === 'number' && Number.isFinite(step.config.tabIndex)
              ? Math.trunc(step.config.tabIndex)
              : NaN;

          if (!Number.isInteger(tabIndex) || tabIndex < 1) {
            throw new Error('TAB_INDEX_INVALID');
          }

          await this.adapter.closeTab(tabIndex);
          nextStepId = step.next;
          break;
        }

        case 'click': {
          const targetValue = step.target?.value?.trim();
          if (!targetValue) {
            options?.onStepStatusChange?.({
              stepId: step.id,
              stepType: step.type,
              status: 'awaiting_input',
            });

            const captured = await this.adapter.captureClickTarget();
            options?.onTargetCaptured?.({
              stepId: step.id,
              stepType: step.type,
              captured,
            });

            options?.onStepStatusChange?.({
              stepId: step.id,
              stepType: step.type,
              status: 'running',
            });

            await this.adapter.click(captured.primary);
          } else if (step.target) {
            await this.adapter.click(step.target);
          } else {
            throw new Error(`Step ${step.id} is missing target`);
          }

          nextStepId = step.next;
          break;
        }

        case 'fill': {
          const targetValue = step.target?.value?.trim();
          const inputValue = this.resolveTemplate(
            typeof step.value === 'string' && step.value.trim() ? step.value : 'Hello World',
            variables
          );

          if (!targetValue) {
            options?.onStepStatusChange?.({
              stepId: step.id,
              stepType: step.type,
              status: 'awaiting_input',
            });

            const captured = await this.adapter.captureClickTarget();
            options?.onTargetCaptured?.({
              stepId: step.id,
              stepType: step.type,
              captured,
            });

            options?.onStepStatusChange?.({
              stepId: step.id,
              stepType: step.type,
              status: 'running',
            });

            await this.adapter.fill(captured.primary, inputValue);
          } else if (step.target) {
            await this.adapter.fill(step.target, inputValue);
          } else {
            throw new Error(`Step ${step.id} is missing target`);
          }

          nextStepId = step.next;
          break;
        }

        case 'upload': {
          const filePath = this.resolveTemplate(
            typeof step.config?.filePath === 'string' ? step.config.filePath.trim() : '',
            variables
          );
          if (!filePath) {
            throw new Error('UPLOAD_FILE_REQUIRED');
          }

          let uploadTarget = step.target;
          if (!step.target?.value?.trim()) {
            options?.onStepStatusChange?.({
              stepId: step.id,
              stepType: step.type,
              status: 'awaiting_input',
            });

            const captured = await this.adapter.captureClickTarget();
            options?.onTargetCaptured?.({
              stepId: step.id,
              stepType: step.type,
              captured,
            });

            options?.onStepStatusChange?.({
              stepId: step.id,
              stepType: step.type,
              status: 'running',
            });

            uploadTarget = captured.primary;
          }

          if (!uploadTarget) {
            throw new Error(`Step ${step.id} is missing target`);
          }

          await this.adapter.uploadFile(uploadTarget, filePath);
          nextStepId = step.next;
          break;
        }
        case 'delay': {
          const durationMs =
            typeof step.duration_ms === 'number' && Number.isFinite(step.duration_ms)
              ? step.duration_ms
              : 1000;
          await new Promise((resolve) => window.setTimeout(resolve, Math.max(durationMs, 0)));
          nextStepId = step.next;
          break;
        }

        case 'wait_for': {
          const timeoutMs =
            typeof step.timeout_ms === 'number' && Number.isFinite(step.timeout_ms)
              ? step.timeout_ms
              : 10000;

          let waitTarget = step.target;
          if (!step.target?.value?.trim()) {
            options?.onStepStatusChange?.({
              stepId: step.id,
              stepType: step.type,
              status: 'awaiting_input',
            });

            const captured = await this.adapter.captureClickTarget();
            options?.onTargetCaptured?.({
              stepId: step.id,
              stepType: step.type,
              captured,
            });

            options?.onStepStatusChange?.({
              stepId: step.id,
              stepType: step.type,
              status: 'running',
            });

            waitTarget = captured.primary;
          }

          if (!waitTarget) {
            throw new Error(`Step ${step.id} is missing target`);
          }

          await this.adapter.waitFor(waitTarget, timeoutMs);
          nextStepId = step.next;
          break;
        }

        case 'condition': {
          const conditionType =
            step.config?.conditionType === 'text_present'
              ? 'text_present'
              : step.config?.conditionType === 'url_contains'
                ? 'url_contains'
                : 'element_visible';

          let matched = false;

          if (conditionType === 'element_visible') {
            let target = step.target;
            if (!step.target?.value?.trim()) {
              options?.onStepStatusChange?.({
                stepId: step.id,
                stepType: step.type,
                status: 'awaiting_input',
              });

              const captured = await this.adapter.captureClickTarget();
              options?.onTargetCaptured?.({
                stepId: step.id,
                stepType: step.type,
                captured,
              });

              options?.onStepStatusChange?.({
                stepId: step.id,
                stepType: step.type,
                status: 'running',
              });

              target = captured.primary;
            }

            if (!target) {
              throw new Error(`Step ${step.id} is missing target`);
            }

            matched = await this.adapter.evaluateCondition({
              type: 'element_visible',
              target,
            });
          } else if (conditionType === 'text_present') {
            const expectedText = this.resolveTemplate(
              typeof step.config?.expectedText === 'string' ? step.config.expectedText.trim() : '',
              variables
            );
            if (!expectedText) {
              throw new Error('CONDITION_TEXT_REQUIRED');
            }

            matched = await this.adapter.evaluateCondition({
              type: 'text_present',
              text: expectedText,
            });
          } else {
            const urlFragment = this.resolveTemplate(
              typeof step.config?.urlFragment === 'string' ? step.config.urlFragment.trim() : '',
              variables
            );
            if (!urlFragment) {
              throw new Error('CONDITION_URL_REQUIRED');
            }

            matched = await this.adapter.evaluateCondition({
              type: 'url_contains',
              value: urlFragment,
            });
          }

          nextStepId = matched ? step.branches?.true ?? null : step.branches?.false ?? null;
          break;
        }

        case 'scroll': {
          const mode =
            step.config?.scrollMode === 'by_viewport'
              ? 'by_viewport'
              : step.config?.scrollMode === 'to_edge'
                ? 'to_edge'
                : 'to_element';

          let scrollTarget = step.target;
          if (mode === 'to_element' && !step.target?.value?.trim()) {
            options?.onStepStatusChange?.({
              stepId: step.id,
              stepType: step.type,
              status: 'awaiting_input',
            });

            const captured = await this.adapter.captureClickTarget();
            options?.onTargetCaptured?.({
              stepId: step.id,
              stepType: step.type,
              captured,
            });

            options?.onStepStatusChange?.({
              stepId: step.id,
              stepType: step.type,
              status: 'running',
            });
            scrollTarget = captured.primary;
          }

          await this.adapter.scroll({
            mode,
            target: mode === 'to_element' ? scrollTarget : undefined,
            direction: step.config?.direction === 'up' ? 'up' : 'down',
            viewportCount:
              typeof step.config?.viewportCount === 'number' && Number.isFinite(step.config.viewportCount)
                ? step.config.viewportCount
                : 1,
            edge: step.config?.edge === 'top' ? 'top' : 'bottom',
            block:
              step.config?.block === 'start' ||
              step.config?.block === 'end' ||
              step.config?.block === 'nearest'
                ? step.config.block
                : 'center',
          });

          nextStepId = step.next;
          break;
        }

        case 'loop': {
          nextStepId = await this.runLoopStep(step, stepMap, screenshots, variables, options);
          break;
        }

        case 'break_loop': {
          options?.onStepStatusChange?.({
            stepId: step.id,
            stepType: step.type,
            status: 'success',
          });
          return { type: 'break_loop' };
        }


        case 'extract_text': {
          const extractType =
            step.config?.extractType === 'href'
              ? 'href'
              : step.config?.extractType === 'value'
                ? 'value'
                : 'text';

          let extractTarget = step.target;
          if (!step.target?.value?.trim()) {
            options?.onStepStatusChange?.({
              stepId: step.id,
              stepType: step.type,
              status: 'awaiting_input',
            });

            const captured = await this.adapter.captureClickTarget();
            options?.onTargetCaptured?.({
              stepId: step.id,
              stepType: step.type,
              captured,
            });

            options?.onStepStatusChange?.({
              stepId: step.id,
              stepType: step.type,
              status: 'running',
            });

            extractTarget = captured.primary;
          }

          if (!extractTarget) {
            throw new Error(`Step ${step.id} is missing target`);
          }

          const extracted = await this.adapter.extract({
            target: extractTarget,
            extractType,
          });

          options?.onDataExtracted?.({
            stepId: step.id,
            stepType: step.type,
            value: extracted.value,
          });
          this.storeStepOutput(step, extracted.value, variables);

          nextStepId = step.next;
          break;
        }
        case 'execute_js': {
          const script = this.resolveTemplate(
            typeof step.script === 'string' ? step.script.trim() : '',
            variables
          );
          if (!script) {
            throw new Error('SCRIPT_REQUIRED');
          }

          const executed = await this.adapter.executeScript(script);
          options?.onDataExtracted?.({
            stepId: step.id,
            stepType: step.type,
            value: executed.value,
          });
          this.storeStepOutput(step, executed.value, variables);

          nextStepId = step.next;
          break;
        }

        case 'screenshot': {
          const mode =
            step.screenshot_mode === 'element'
              ? 'element'
              : step.screenshot_mode === 'full_page'
                ? 'full_page'
                : 'viewport';
          let screenshotTarget = step.target;

          if (mode === 'element' && !step.target?.value?.trim()) {
            options?.onStepStatusChange?.({
              stepId: step.id,
              stepType: step.type,
              status: 'awaiting_input',
            });

            const captured = await this.adapter.captureClickTarget();
            options?.onTargetCaptured?.({
              stepId: step.id,
              stepType: step.type,
              captured,
            });

            options?.onStepStatusChange?.({
              stepId: step.id,
              stepType: step.type,
              status: 'running',
            });

            screenshotTarget = captured.primary;
          }

          const data = await this.adapter.screenshot({
            fullPage: mode === 'full_page',
            target: mode === 'element' ? screenshotTarget : undefined,
          });
          screenshots.push(data);
          options?.onScreenshotCaptured?.({
            stepId: step.id,
            stepType: step.type,
            screenshot: data,
          });
          nextStepId = step.next;
          break;
        }

        default:
          throw new Error(`Step type is not implemented yet: ${step.type}`);
      }

      options?.onStepStatusChange?.({
        stepId: step.id,
        stepType: step.type,
        status: 'success',
      });

      return {
        type: 'next',
        nextStepId,
      };
    } catch (error) {
      options?.onStepStatusChange?.({
        stepId: step.id,
        stepType: step.type,
        status: 'failed',
        error: error instanceof Error ? error.message : String(error),
      });
      throw error;
    }
  }

  private async runLoopStep(
    step: RpaWorkflowStep,
    stepMap: Map<string, RpaWorkflowStep>,
    screenshots: string[],
    variables: Record<string, unknown>,
    options?: RpaRunnerOptions
  ): Promise<string | null> {
    const iterations =
      typeof step.config?.iterations === 'number' && Number.isFinite(step.config.iterations)
        ? Math.max(1, Math.floor(step.config.iterations))
        : 3;

    const loopChildren = Array.from(stepMap.values()).filter((candidate) => candidate.ui?.parent_id === step.id);
    if (loopChildren.length === 0) {
      return step.next;
    }

    const loopChildIds = new Set(loopChildren.map((candidate) => candidate.id));
    const incoming = new Set<string>();

    for (const child of loopChildren) {
      if (child.next && loopChildIds.has(child.next)) {
        incoming.add(child.next);
      }

      if (child.branches) {
        for (const target of Object.values(child.branches)) {
          if (target && loopChildIds.has(target)) {
            incoming.add(target);
          }
        }
      }
    }

    const entryStep =
      loopChildren
        .filter((candidate) => !incoming.has(candidate.id))
        .sort((a, b) => {
          const ay = a.ui?.position?.y ?? 0;
          const by = b.ui?.position?.y ?? 0;
          if (ay !== by) {
            return ay - by;
          }

          const ax = a.ui?.position?.x ?? 0;
          const bx = b.ui?.position?.x ?? 0;
          return ax - bx;
        })[0] ?? loopChildren[0];

    for (let index = 0; index < iterations; index += 1) {
      options?.onStepStatusChange?.({
        stepId: step.id,
        stepType: step.type,
        status: 'running',
        loopIteration: index + 1,
        loopTotal: iterations,
      });

      let currentStepId: string | null = entryStep.id;
      const visited = new Set<string>();

      while (currentStepId && loopChildIds.has(currentStepId)) {
        if (visited.has(currentStepId)) {
          throw new Error('LOOP_BODY_CYCLE');
        }

        visited.add(currentStepId);
        const childStep = stepMap.get(currentStepId);
        if (!childStep) {
          throw new Error(`Unknown workflow step: ${currentStepId}`);
        }

        const result = await this.runStep(childStep, stepMap, screenshots, variables, options);
        if (result.type === 'break_loop') {
          return step.next;
        }

        currentStepId =
          result.nextStepId && loopChildIds.has(result.nextStepId) ? result.nextStepId : null;
      }
    }

    return step.next;
  }

  private resolveTemplate(value: string, variables: Record<string, unknown>): string {
    if (!value.includes('{{')) {
      return value;
    }

    return value.replace(/\{\{\s*([^}]+?)\s*\}\}/g, (_, expression: string) => {
      const resolved = this.resolveVariableExpression(expression.trim(), variables);
      if (resolved == null) {
        return '';
      }

      if (typeof resolved === 'string') {
        return resolved;
      }

      if (typeof resolved === 'number' || typeof resolved === 'boolean') {
        return String(resolved);
      }

      return JSON.stringify(resolved);
    });
  }

  private storeStepOutput(step: RpaWorkflowStep, value: unknown, variables: Record<string, unknown>): void {
    const outputKey = typeof step.output === 'string' ? step.output.trim() : '';
    if (!outputKey) {
      return;
    }

    variables[outputKey] = value;
  }

  private resolveVariableExpression(expression: string, variables: Record<string, unknown>): unknown {
    const tokens = this.parseVariablePath(expression);
    if (tokens.length === 0) {
      return '';
    }

    let current = this.normalizeVariableValue(variables[String(tokens[0])]);
    for (let index = 1; index < tokens.length; index += 1) {
      if (current == null) {
        return '';
      }

      const token = tokens[index];
      if (Array.isArray(current) && typeof token === 'number') {
        current = this.normalizeVariableValue(current[token]);
        continue;
      }

      if (typeof current === 'object' && current !== null && token in current) {
        current = this.normalizeVariableValue(current[String(token)]);
        continue;
      }

      if ((typeof current === 'string' || Array.isArray(current)) && token === 'length') {
        return current.length;
      }

      return '';
    }

    return current;
  }

  private initializeWorkflowVariables(workflow: RpaWorkflowSchema): Record<string, unknown> {
    const variables: Record<string, unknown> = {};

    for (const variable of workflow.variables ?? []) {
      const name = typeof variable?.name === 'string' ? variable.name.trim() : '';
      if (!name) {
        continue;
      }

      variables[name] = variable.value;
    }

    return variables;
  }

  private parseVariablePath(expression: string): Array<string | number> {
    const tokens: Array<string | number> = [];
    const pattern = /([A-Za-z_][\w-]*)|\[(\d+)\]/g;
    for (const match of expression.matchAll(pattern)) {
      if (match[1]) {
        tokens.push(match[1]);
      } else if (match[2]) {
        tokens.push(Number(match[2]));
      }
    }
    return tokens;
  }

  private normalizeVariableValue(value: unknown): unknown {
    if (typeof value !== 'string') {
      return value;
    }

    const trimmed = value.trim();
    if (!trimmed) {
      return value;
    }

    if ((trimmed.startsWith('[') && trimmed.endsWith(']')) || (trimmed.startsWith('{') && trimmed.endsWith('}'))) {
      try {
        return JSON.parse(trimmed);
      } catch {
        return value;
      }
    }

    return value;
  }
}
