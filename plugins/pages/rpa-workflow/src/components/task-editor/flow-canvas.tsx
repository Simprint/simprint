import { useCallback, useMemo, useRef, useEffect, useState } from 'react';
import type { MouseEvent as ReactMouseEvent } from 'react';
import { useTranslation } from 'react-i18next';
import type { TFunction } from 'react-i18next';
import {
  ReactFlow,
  Background,
  Controls,
  Panel,
  useNodesState,
  useEdgesState,
  BackgroundVariant,
  MarkerType,
  ReactFlowProvider,
  useReactFlow,
} from '@xyflow/react';
import { Trash2 } from 'lucide-react';
import type {
  Node,
  Edge,
  Connection,
  NodeTypes,
  ReactFlowInstance,
  OnConnectEnd,
  CoordinateExtent,
} from '@xyflow/react';
import '@xyflow/react/dist/style.css';

import { StepNode, StartNode, EndNode, LoopNode } from './nodes';
import {
  DEFAULT_STEP_HEIGHT,
  DEFAULT_STEP_WIDTH,
  LOOP_REGION_CONTENT_BOTTOM,
  LOOP_REGION_CONTENT_TOP,
  LOOP_REGION_PADDING_X,
  LOOP_REGION_PADDING_RIGHT,
  getLoopAutoDimensions,
  getLoopDimensions,
  getLoopExpandedHeightForChild,
  getLoopExpandedWidthForChild,
  getLoopHeightFromChildren,
  getLoopWidthFromChildren,
} from './loop-layout';
import type { ComponentItem } from './component-panel';
import type { RpaRunnerStepStatus } from '../../runtime/runner';

export interface FlowStep {
  id: string;
  type: string;
  name: string;
  config: Record<string, unknown>;
  enabled: boolean;
  isStart?: boolean;
  nextStepId?: string | null;
  position?: { x: number; y: number };
  parentLoopId?: string | null;
}

interface FlowCanvasProps {
  steps: FlowStep[];
  selectedStepId: string | null;
  onSelectStep: (id: string | null) => void;
  onDeleteStep: (id: string) => void;
  onUpdateSteps: (steps: FlowStep[]) => void;
  onDropComponent: (
    component: ComponentItem,
    position: { x: number; y: number },
    sourceId?: string | null,
    sourceHandle?: string | null,
    parentLoopId?: string | null
  ) => void;
  onClearSteps?: () => void;
  clearStepsDisabled?: boolean;
  draggingComponent: ComponentItem | null;
  runStatus?: 'idle' | 'starting' | 'running' | 'success' | 'failed' | 'stopping' | 'stopped';
  stepStatuses?: Record<string, RpaRunnerStepStatus>;
  stepErrors?: Record<string, string>;
  stepLoopProgress?: Record<string, { current?: number; total?: number }>;
}

const nodeTypes: NodeTypes = {
  stepNode: StepNode,
  startNode: StartNode,
  endNode: EndNode,
  loopNode: LoopNode,
};

interface SpecialNodePositions {
  start: { x: number; y: number };
  end: { x: number; y: number };
}

function isLoopStep(step: FlowStep): boolean {
  return step.type === 'loop';
}

function getLoopRect(step: FlowStep) {
  const position = step.position ?? { x: 200, y: 120 };
  const { width, height } = getLoopDimensions(step);
  return {
    left: position.x,
    top: position.y,
    right: position.x + width,
    bottom: position.y + height,
    width,
    height,
  };
}

function clamp(value: number, min: number, max: number): number {
  return Math.min(Math.max(value, min), max);
}

function toLoopChildPosition(loopStep: FlowStep, absolutePosition: { x: number; y: number }) {
  const loopPosition = loopStep.position ?? { x: 0, y: 0 };
  const { width, height } = getLoopDimensions(loopStep);
  const minX = LOOP_REGION_PADDING_X;
  const maxX = Math.max(LOOP_REGION_PADDING_X, width - LOOP_REGION_PADDING_RIGHT);
  const minY = LOOP_REGION_CONTENT_TOP;
  const maxY = Math.max(minY, height - DEFAULT_STEP_HEIGHT - LOOP_REGION_CONTENT_BOTTOM);

  return {
    x: clamp(absolutePosition.x - loopPosition.x, minX, maxX),
    y: clamp(absolutePosition.y - loopPosition.y, minY, maxY),
  };
}

function getLoopChildExtent(loopStep: FlowStep): CoordinateExtent {
  const { width, height } = getLoopDimensions(loopStep);
  const maxX = Math.max(LOOP_REGION_PADDING_X, width - LOOP_REGION_PADDING_RIGHT);
  const maxY = Math.max(LOOP_REGION_CONTENT_TOP, height - LOOP_REGION_CONTENT_BOTTOM);

  return [
    [LOOP_REGION_PADDING_X, LOOP_REGION_CONTENT_TOP],
    [maxX, maxY],
  ];
}

function findContainingLoopStep(
  steps: FlowStep[],
  position: { x: number; y: number },
  excludeLoopId?: string | null
): FlowStep | null {
  const loopSteps = steps.filter((step) => isLoopStep(step) && step.id !== excludeLoopId);

  for (let index = loopSteps.length - 1; index >= 0; index -= 1) {
    const loopStep = loopSteps[index];
    const rect = getLoopRect(loopStep);
    if (
      position.x >= rect.left &&
      position.x <= rect.right &&
      position.y >= rect.top &&
      position.y <= rect.bottom
    ) {
      return loopStep;
    }
  }

  return null;
}


function readString(value: unknown): string {
  return typeof value === 'string' ? value.trim() : '';
}

function getConditionBranches(step: FlowStep): Record<'true' | 'false', string | null> {
  const branches = step.config.branches as Record<string, unknown> | undefined;
  return {
    true: typeof branches?.true === 'string' ? branches.true : null,
    false: typeof branches?.false === 'string' ? branches.false : null,
  };
}

function getNextStepIds(step: FlowStep): string[] {
  if (step.type === 'condition') {
    return Object.values(getConditionBranches(step)).filter((value): value is string => Boolean(value));
  }

  if (step.type === 'break_loop') {
    return [];
  }

  return step.nextStepId ? [step.nextStepId] : [];
}

function summarizeCondition(step: FlowStep, t: TFunction<'rpa'>): string {
  const conditionType = readString(step.config.conditionType) || 'element_visible';
  switch (conditionType) {
    case 'text_present': {
      const expectedText = readString(step.config.expectedText);
      return expectedText || t('editor.condition.textPlaceholder', { defaultValue: '请输入目标文本' });
    }
    case 'url_contains': {
      const urlFragment = readString(step.config.urlFragment);
      return urlFragment || t('editor.condition.urlPlaceholder', { defaultValue: '请输入地址片段' });
    }
    default: {
      const selector = readString(step.config.selector);
      return selector || t('editor.condition.elementHint', { defaultValue: '运行后在打开的浏览器中选择判断元素' });
    }
  }
}

function summarizeStep(step: FlowStep, t: TFunction<'rpa'>): string {
  switch (step.type) {
    case 'navigate': {
      const url = readString(step.config.url);
      return url || t('editor.stepSubtitleDefaults.navigate');
    }
    case 'select_tab': {
      const tabIndex =
        typeof step.config.tabIndex === 'number' && Number.isFinite(step.config.tabIndex)
          ? Math.trunc(step.config.tabIndex)
          : 1;
      return t('editor.stepSubtitleDefaults.selectTab', {
        defaultValue: '切换到第 {{index}} 个标签页',
        index: Math.max(1, tabIndex),
      });
    }
    case 'close_tab': {
      const tabIndex =
        typeof step.config.tabIndex === 'number' && Number.isFinite(step.config.tabIndex)
          ? Math.trunc(step.config.tabIndex)
          : 1;
      return t('editor.stepSubtitleDefaults.closeTab', {
        defaultValue: '关闭第 {{index}} 个标签页',
        index: Math.max(1, tabIndex),
      });
    }
    case 'click': {
      const selector = readString(step.config.selector);
      return selector ? t('editor.stepSubtitleDefaults.clickReady') : t('editor.stepSubtitleDefaults.clickCapture');
    }
    case 'input': {
      const selector = readString(step.config.selector);
      if (!selector) {
        return t('editor.stepSubtitleDefaults.inputCapture');
      }

      const text = readString(step.config.text);
      return text || t('editor.stepSubtitleDefaults.inputDefault');
    }
    case 'scroll':
      return t('editor.stepSubtitleDefaults.scroll');
    case 'wait':
      if (step.config.waitType === 'time') {
        const duration = typeof step.config.duration === 'number' ? step.config.duration : 1000;
        return `${duration} ms`;
      }
      return readString(step.config.selector) || t('editor.stepSubtitleDefaults.waitElementCapture', { defaultValue: '运行后在打开的浏览器中选择等待目标' });
    case 'condition':
      return summarizeCondition(step, t);
    case 'extract': {
      const previewValue = readString(step.config.previewValue);
      return previewValue || readString(step.config.selector) || t('editor.stepSubtitleDefaults.extract');
    }
    case 'download':
      return t('editor.stepSubtitleDefaults.download');
    case 'upload': {
      const filePath = readString(step.config.filePath);
      if (!filePath) {
        return t('editor.stepSubtitleDefaults.upload');
      }

      const normalizedPath = filePath.replace(/\\/g, '/');
      const fileName = normalizedPath.split('/').pop() ?? normalizedPath;
      return fileName || t('editor.stepSubtitleDefaults.upload');
    }
    case 'variable':
      return t('editor.stepSubtitleDefaults.variable');
    case 'email':
      return t('editor.stepSubtitleDefaults.email');
    case 'api':
      return t('editor.stepSubtitleDefaults.api');
    case 'script': {
      const previewValue = readString(step.config.previewValue);
      if (previewValue) {
        return previewValue;
      }
      const script = readString(step.config.script).split('\n')[0] ?? '';
      return script || t('editor.stepSubtitleDefaults.script');
    }
    case 'loop':
      return t('editor.stepSubtitleDefaults.loop');
    case 'break_loop':
      return t('editor.stepSubtitleDefaults.breakLoop', {
        defaultValue: '满足条件时立即退出当前循环',
      });
    case 'notify':
      return t('editor.stepSubtitleDefaults.notify');
    case 'screenshot': {
      const mode = readString(step.config.captureMode) || 'viewport';
      if (mode === 'element') {
        return readString(step.config.selector) || t('editor.stepSubtitleDefaults.screenshotElementCapture', { defaultValue: '运行后在打开的浏览器中选择截图区域' });
      }
      if (mode === 'full_page') {
        return t('editor.stepSubtitleDefaults.screenshotFullPage', { defaultValue: '运行后截取整页内容' });
      }
      return t('editor.stepSubtitleDefaults.screenshotViewport', { defaultValue: '运行后截取当前视口' });
    }
    default:
      return '';
  }
}

function createBaseEdge(
  source: string,
  target: string,
  options?: { label?: string; sourceHandle?: string | null }
): Edge {
  const labelPart = options?.label ? `-${options.label}` : '';
  const handlePart = options?.sourceHandle ? `-${options.sourceHandle}` : '';
  return {
    id: `${source}${handlePart}-to-${target}${labelPart}`,
    source,
    target,
    sourceHandle: options?.sourceHandle ?? undefined,
    label: options?.label,
    type: 'smoothstep',
    markerEnd: { type: MarkerType.ArrowClosed },
    style: { stroke: '#64748b', strokeWidth: 2 },
    labelStyle: { fill: '#64748b', fontSize: 10, fontWeight: 600 },
    labelBgStyle: { fill: 'hsl(var(--background))', fillOpacity: 0.92 },
  };
}

function getReachableStepIds(steps: FlowStep[]): Set<string> {
  const stepMap = new Map(steps.map((step) => [step.id, step]));
  const visited = new Set<string>();
  const startStep = steps.find((step) => step.isStart);
  const stack = startStep ? [startStep.id] : [];

  while (stack.length > 0) {
    const currentStepId = stack.pop();
    if (!currentStepId || visited.has(currentStepId)) {
      continue;
    }

    visited.add(currentStepId);
    const currentStep = stepMap.get(currentStepId);
    if (!currentStep) {
      continue;
    }

    for (const nextStepId of getNextStepIds(currentStep)) {
      if (!visited.has(nextStepId)) {
        stack.push(nextStepId);
      }
    }
  }

  return visited;
}

function buildGraphEdges(steps: FlowStep[], t: TFunction<'rpa'>): Edge[] {
  const edges: Edge[] = [];
  const reachableStepIds = getReachableStepIds(steps);
  const startStep = steps.find((step) => step.isStart);

  if (startStep) {
    edges.push(createBaseEdge('start', startStep.id));
  }

  for (const step of steps) {
    if (step.type === 'condition') {
      const branches = getConditionBranches(step);
      const trueLabel = t('editor.condition.trueBranch', { defaultValue: '是' });
      const falseLabel = t('editor.condition.falseBranch', { defaultValue: '否' });

      if (branches.true) {
        edges.push(createBaseEdge(step.id, branches.true, { label: trueLabel, sourceHandle: 'branch-true' }));
      } else if (reachableStepIds.has(step.id)) {
        edges.push(createBaseEdge(step.id, 'end', { label: trueLabel, sourceHandle: 'branch-true' }));
      }

      if (branches.false) {
        edges.push(createBaseEdge(step.id, branches.false, { label: falseLabel, sourceHandle: 'branch-false' }));
      } else if (reachableStepIds.has(step.id)) {
        edges.push(createBaseEdge(step.id, 'end', { label: falseLabel, sourceHandle: 'branch-false' }));
      }

      continue;
    }

    if (step.type === 'break_loop') {
      continue;
    }

    if (step.nextStepId) {
      edges.push(createBaseEdge(step.id, step.nextStepId));
      continue;
    }

    if (reachableStepIds.has(step.id)) {
      edges.push(createBaseEdge(step.id, 'end'));
    }
  }

  return edges;
}

function styleEdgeByRunState(
  edge: Edge,
  stepStatuses: Record<string, RpaRunnerStepStatus>,
  runStatus: FlowCanvasProps['runStatus']
): Edge {
  const sourceStatus =
    edge.source === 'start'
      ? runStatus === 'starting' || runStatus === 'running' || runStatus === 'success'
        ? 'success'
        : 'pending'
      : stepStatuses[edge.source] ?? 'pending';
  const targetStatus =
    edge.target === 'end'
      ? runStatus === 'success'
        ? 'success'
        : runStatus === 'failed'
          ? 'failed'
          : 'pending'
      : stepStatuses[edge.target] ?? 'pending';

  if (targetStatus === 'failed') {
    return {
      ...edge,
      animated: false,
      style: { stroke: '#ef4444', strokeWidth: 2.5 },
      markerEnd: { type: MarkerType.ArrowClosed, color: '#ef4444' },
      labelStyle: { ...(edge.labelStyle ?? {}), fill: '#ef4444' },
    };
  }

  if (targetStatus === 'running' || targetStatus === 'awaiting_input') {
    const color = targetStatus === 'awaiting_input' ? '#d97706' : '#0ea5e9';
    return {
      ...edge,
      animated: true,
      style: { stroke: color, strokeWidth: 2.5 },
      markerEnd: { type: MarkerType.ArrowClosed, color },
      labelStyle: { ...(edge.labelStyle ?? {}), fill: color },
    };
  }

  if (sourceStatus === 'success' && targetStatus === 'success') {
    return {
      ...edge,
      animated: false,
      style: { stroke: '#10b981', strokeWidth: 2.5 },
      markerEnd: { type: MarkerType.ArrowClosed, color: '#10b981' },
      labelStyle: { ...(edge.labelStyle ?? {}), fill: '#10b981' },
    };
  }

  return {
    ...edge,
    animated: false,
    style: { stroke: '#64748b', strokeWidth: 2 },
    markerEnd: { type: MarkerType.ArrowClosed, color: '#64748b' },
    labelStyle: { ...(edge.labelStyle ?? {}), fill: '#64748b' },
  };
}

function stepsToNodes(
  steps: FlowStep[],
  onDelete: (id: string) => void,
  specialPositions: SpecialNodePositions,
  t: TFunction<'rpa'>,
  runStatus: FlowCanvasProps['runStatus'],
  stepStatuses: Record<string, RpaRunnerStepStatus>,
  stepErrors: Record<string, string>,
  stepLoopProgress: Record<string, { current?: number; total?: number }>
): Node[] {
  const nodes: Node[] = [];
  const loopStepIds = new Set(steps.filter((step) => isLoopStep(step)).map((step) => step.id));

  nodes.push({
    id: 'start',
    type: 'startNode',
    position: specialPositions.start,
    data: { runStatus },
  });

  steps
    .filter((step) => isLoopStep(step))
    .forEach((step) => {
      const { width, height } = getLoopDimensions(step);
      nodes.push({
        id: step.id,
        type: 'loopNode',
        position: step.position || { x: 200, y: 100 },
        style: { width, height },
        data: {
          label: step.name,
          subtitle: summarizeStep(step, t),
          enabled: step.enabled,
          iterations:
            typeof step.config.iterations === 'number' && Number.isFinite(step.config.iterations)
              ? step.config.iterations
              : 3,
          currentIteration: stepLoopProgress[step.id]?.current,
          totalIterations: stepLoopProgress[step.id]?.total,
          runStatus: stepStatuses[step.id] ?? 'pending',
          errorSummary: stepErrors[step.id] ?? '',
          childCount: steps.filter((candidate) => candidate.parentLoopId === step.id).length,
          onDelete,
        } as unknown as Record<string, unknown>,
      });
    });

  steps
    .filter((step) => !isLoopStep(step))
    .forEach((step, index) => {
      const parentLoop = step.parentLoopId
        ? steps.find((candidate) => candidate.id === step.parentLoopId && isLoopStep(candidate)) ?? null
        : null;
      const hasParentLoop = Boolean(parentLoop) && loopStepIds.has(parentLoop.id);
      nodes.push({
        id: step.id,
        type: 'stepNode',
        position: step.position || { x: 200, y: 100 + index * 100 },
        parentId: hasParentLoop ? step.parentLoopId ?? undefined : undefined,
        extent: hasParentLoop && parentLoop ? getLoopChildExtent(parentLoop) : undefined,
        data: {
          label: step.name,
          subtitle: summarizeStep(step, t),
          type: step.type,
          enabled: step.enabled,
          runStatus: stepStatuses[step.id] ?? 'pending',
          errorSummary: stepErrors[step.id] ?? '',
          onDelete,
        } as unknown as Record<string, unknown>,
      });
    });

  nodes.push({
    id: 'end',
    type: 'endNode',
    position: specialPositions.end,
    data: { runStatus },
  });

  return nodes;
}

function FlowCanvasInner({
  steps,
  selectedStepId,
  onSelectStep,
  onDeleteStep,
  onUpdateSteps,
  onDropComponent,
  onClearSteps,
  clearStepsDisabled = false,
  draggingComponent,
  runStatus = 'idle',
  stepStatuses = {},
  stepErrors = {},
  stepLoopProgress = {},
}: FlowCanvasProps) {
  const { t } = useTranslation('rpa');
  const reactFlowWrapper = useRef<HTMLDivElement>(null);
  const stepsRef = useRef(steps);
  const [reactFlowInstance, setReactFlowInstance] = useState<ReactFlowInstance | null>(null);
  const { screenToFlowPosition } = useReactFlow();

  const specialPositionsRef = useRef<SpecialNodePositions>({
    start: { x: 250, y: 0 },
    end: { x: 250, y: 200 },
  });

  const initialSpecialPositions: SpecialNodePositions = {
    start: { x: 250, y: 0 },
    end: { x: 250, y: 200 },
  };

  const [nodes, setNodes, onNodesChange] = useNodesState(
    stepsToNodes(
      steps,
      onDeleteStep,
      initialSpecialPositions,
      t,
      runStatus,
      stepStatuses,
      stepErrors,
      stepLoopProgress
    )
  );
  const [edges, setEdges, onEdgesChange] = useEdgesState<Edge>(buildGraphEdges(steps, t));

  useEffect(() => {
    stepsRef.current = steps;
  }, [steps]);


  useEffect(() => {
    setNodes(
      stepsToNodes(
        steps,
        onDeleteStep,
        specialPositionsRef.current,
        t,
        runStatus,
        stepStatuses,
        stepErrors,
        stepLoopProgress
      )
    );
    setEdges(buildGraphEdges(steps, t));
  }, [steps, onDeleteStep, setNodes, setEdges, t, runStatus, stepStatuses, stepErrors, stepLoopProgress]);


const styledEdges = useMemo(
    () => edges.map((edge) => styleEdgeByRunState(edge, stepStatuses, runStatus)),
    [edges, runStatus, stepStatuses]
  );

  const onConnect = useCallback(
    (params: Connection) => {
      if (!params.source || !params.target) {
        return;
      }

      const sourceId = params.source;
      const targetId = params.target;
      const sourceHandle = params.sourceHandle ?? null;
      const currentStartStep = steps.find((step) => step.isStart);
      const sourceStep = steps.find((step) => step.id === sourceId);

      if (sourceId === 'start') {
        if (currentStartStep && currentStartStep.id !== targetId) {
          return;
        }
      } else if (!sourceStep) {
        return;
      } else if (sourceStep.type === 'break_loop') {
        return;
      } else if (sourceStep.type === 'condition') {
        const branchKey = sourceHandle === 'branch-false' ? 'false' : 'true';
        const branches = getConditionBranches(sourceStep);
        if (branches[branchKey as 'true' | 'false'] && branches[branchKey as 'true' | 'false'] !== targetId) {
          return;
        }
      } else if (sourceStep.nextStepId && sourceStep.nextStepId !== targetId) {
        return;
      }

      onUpdateSteps(
        steps.map((step) => {
          if (sourceId === 'start') {
            return { ...step, isStart: step.id === targetId };
          }

          if (step.id !== sourceId) {
            return step;
          }

          if (step.type === 'condition') {
            const branchKey = sourceHandle === 'branch-false' ? 'false' : 'true';
            const currentBranches = (step.config.branches as Record<string, unknown> | undefined) ?? {};
            return {
              ...step,
              config: {
                ...step.config,
                branches: {
                  ...currentBranches,
                  [branchKey]: targetId,
                },
              },
            };
          }

          return { ...step, nextStepId: targetId };
        })
      );
    },
    [onUpdateSteps, steps]
  );

  const onConnectEnd: OnConnectEnd = useCallback(
    (event, connectionState) => {
      if (!connectionState.isValid && connectionState.fromNode) {
        const { clientX, clientY } =
          event instanceof MouseEvent ? event : (event as TouchEvent).changedTouches[0];

        const position = screenToFlowPosition({ x: clientX, y: clientY });
        const sourceHandle = (connectionState as { fromHandle?: { id?: string | null } }).fromHandle?.id ?? null;
        const sourceStep = steps.find((step) => step.id === connectionState.fromNode.id);
        const sourceParentLoop = sourceStep?.parentLoopId
          ? steps.find((step) => step.id === sourceStep.parentLoopId && isLoopStep(step)) ?? null
          : null;
        const parentLoop = sourceParentLoop ?? findContainingLoopStep(steps, position, null);
        const normalizedPosition = parentLoop
          ? toLoopChildPosition(parentLoop, position)
          : position;

        if (draggingComponent) {
          onDropComponent(
            draggingComponent,
            normalizedPosition,
            connectionState.fromNode.id,
            sourceHandle,
            parentLoop?.id ?? null
          );
        } else {
          const defaultComponent: Omit<ComponentItem, 'icon'> = {
            id: 'click',
            type: 'click',
            name: '点击元素',
            category: 'browser',
            description: '点击页面上的元素',
          };
          onDropComponent(
            defaultComponent as ComponentItem,
            normalizedPosition,
            connectionState.fromNode.id,
            sourceHandle,
            parentLoop?.id ?? null
          );
        }
      }
    },
    [draggingComponent, onDropComponent, screenToFlowPosition, steps]
  );

  const onNodeClick = useCallback(
    (_: ReactMouseEvent, node: Node) => {
      if (node.id !== 'start' && node.id !== 'end') {
        onSelectStep(node.id);
      }
    },
    [onSelectStep]
  );

  const onPaneClick = useCallback(() => {
    onSelectStep(null);
  }, [onSelectStep]);

  const onNodeDragStop = useCallback(
    (event: ReactMouseEvent, node: Node) => {
      if (node.id === 'start' || node.id === 'end') {
        specialPositionsRef.current = {
          ...specialPositionsRef.current,
          [node.id]: node.position,
        };
        return;
      }

      const draggedStep = steps.find((step) => step.id === node.id);
      if (!draggedStep) {
        return;
      }

      if (isLoopStep(draggedStep)) {
        const updatedSteps = steps.map((step) => {
          if (step.id === node.id) {
            return { ...step, position: node.position };
          }
          return step;
        });
        const recalculatedSteps = updatedSteps.map((step) => {
        if (!isLoopStep(step)) {
          return step;
        }

        const loopChildren = updatedSteps.filter((candidate) => candidate.parentLoopId === step.id);
        const width = getLoopWidthFromChildren(step, loopChildren);
        const height = getLoopHeightFromChildren(step, loopChildren);

        return {
          ...step,
          config: {
            ...step.config,
            loopWidth: width,
            loopHeight: height,
          },
        };
      });

      onUpdateSteps(recalculatedSteps);
        return;
      }

      const absolutePosition =
        'positionAbsolute' in node && node.positionAbsolute
          ? node.positionAbsolute
          : draggedStep.parentLoopId
            ? {
                x: (steps.find((step) => step.id === draggedStep.parentLoopId)?.position?.x ?? 0) + node.position.x,
                y: (steps.find((step) => step.id === draggedStep.parentLoopId)?.position?.y ?? 0) + node.position.y,
              }
            : node.position;

      const hitPosition = {
        x: absolutePosition.x + DEFAULT_STEP_WIDTH / 2,
        y: absolutePosition.y + DEFAULT_STEP_HEIGHT / 2,
      };
      const sourceLoop = draggedStep.parentLoopId
        ? steps.find((step) => step.id === draggedStep.parentLoopId && isLoopStep(step)) ?? null
        : null;
      const targetLoop = findContainingLoopStep(steps, hitPosition, null) ?? sourceLoop;
      const effectiveTargetLoop = draggedStep.type === 'break_loop' ? targetLoop ?? sourceLoop : targetLoop;

      const targetLoopChildCount = effectiveTargetLoop
        ? steps.filter((step) => step.parentLoopId === effectiveTargetLoop.id && step.id !== node.id).length + 1
        : 0;
      const pointerFlowPosition = screenToFlowPosition({ x: event.clientX, y: event.clientY });
      const relativeX = effectiveTargetLoop
        ? Math.max(LOOP_REGION_PADDING_X, pointerFlowPosition.x - (effectiveTargetLoop.position?.x ?? 0) - DEFAULT_STEP_WIDTH / 2)
        : 0;
      const relativeY = effectiveTargetLoop
        ? Math.max(LOOP_REGION_CONTENT_TOP, pointerFlowPosition.y - (effectiveTargetLoop.position?.y ?? 0) - DEFAULT_STEP_HEIGHT / 2)
        : 0;

      const targetLoopNextDimensions = effectiveTargetLoop
        ? (() => {
            const dimensions = getLoopAutoDimensions(effectiveTargetLoop, targetLoopChildCount);
            return {
              width: getLoopExpandedWidthForChild(effectiveTargetLoop, relativeX),
              height: Math.max(dimensions.height, getLoopExpandedHeightForChild(effectiveTargetLoop, relativeY)),
            };
          })()
        : null;
      const desiredAbsolutePosition = effectiveTargetLoop
        ? {
            x: pointerFlowPosition.x - DEFAULT_STEP_WIDTH / 2,
            y: pointerFlowPosition.y - DEFAULT_STEP_HEIGHT / 2,
          }
        : absolutePosition;

      const updatedSteps = steps.map((step) => {
        if (effectiveTargetLoop && step.id === effectiveTargetLoop.id && targetLoopNextDimensions) {
          return {
            ...step,
            config: {
              ...step.config,
              loopWidth: targetLoopNextDimensions.width,
              loopHeight: targetLoopNextDimensions.height,
            },
          };
        }

        if (step.id !== node.id) {
          return step;
        }

        if (effectiveTargetLoop && targetLoopNextDimensions) {
          return {
            ...step,
            parentLoopId: effectiveTargetLoop.id,
            position: toLoopChildPosition(
              {
                ...effectiveTargetLoop,
                config: {
                  ...effectiveTargetLoop.config,
                  loopWidth: targetLoopNextDimensions.width,
                  loopHeight: targetLoopNextDimensions.height,
                },
              },
              desiredAbsolutePosition
            ),
          };
        }

        if (draggedStep.type === 'break_loop') {
          return step;
        }

        return {
          ...step,
          parentLoopId: null,
          position: absolutePosition,
        };
      });

      const recalculatedSteps = updatedSteps.map((step) => {
        if (!isLoopStep(step)) {
          return step;
        }

        const loopChildren = updatedSteps.filter((candidate) => candidate.parentLoopId === step.id);
        const width = getLoopWidthFromChildren(step, loopChildren);
        const height = getLoopHeightFromChildren(step, loopChildren);

        return {
          ...step,
          config: {
            ...step.config,
            loopWidth: width,
            loopHeight: height,
          },
        };
      });

      onUpdateSteps(recalculatedSteps);
    },
    [screenToFlowPosition, steps, onUpdateSteps]
  );

  const handleCanvasClick = useCallback(
    (event: ReactMouseEvent) => {
      if (!draggingComponent || !reactFlowInstance) return;

      const absolutePosition = reactFlowInstance.screenToFlowPosition({
        x: event.clientX,
        y: event.clientY,
      });
      const parentLoop = draggingComponent.type === 'loop' ? null : findContainingLoopStep(steps, absolutePosition, null);
      const position = parentLoop ? toLoopChildPosition(parentLoop, absolutePosition) : absolutePosition;

      onDropComponent(draggingComponent, position, undefined, undefined, parentLoop?.id ?? null);
    },
    [draggingComponent, onDropComponent, reactFlowInstance, steps]
  );

  const nodesWithSelection = useMemo(
    () => nodes.map((node) => ({ ...node, selected: node.id === selectedStepId })),
    [nodes, selectedStepId]
  );

  return (
    <div className="flex-1 bg-background flex flex-col min-h-0 overflow-hidden">
      <div className="px-4 py-2 border-b border-border bg-background flex items-center justify-between shrink-0">
        <h3 className="text-xs font-semibold text-foreground">{t('editor.flowCanvas')}</h3>
        <span className="text-[10px] text-muted-foreground">{t('editor.stepCount', { count: steps.length })}</span>
      </div>

      <div
        ref={reactFlowWrapper}
        className={`flex-1 min-h-0 relative ${draggingComponent ? 'ring-2 ring-primary ring-inset cursor-copy' : ''}`}
        onClick={draggingComponent ? handleCanvasClick : undefined}
      >
        <ReactFlow
          nodes={nodesWithSelection}
          edges={styledEdges}
          onNodesChange={onNodesChange}
          onEdgesChange={onEdgesChange}
          onConnect={onConnect}
          onConnectEnd={onConnectEnd}
          onNodeClick={onNodeClick}
          onPaneClick={onPaneClick}
          onNodeDragStop={onNodeDragStop}
          onInit={setReactFlowInstance}
          nodeTypes={nodeTypes}
          defaultViewport={{ x: 0, y: 0, zoom: 1 }}
          minZoom={0.25}
          maxZoom={2}
          defaultEdgeOptions={{
            type: 'smoothstep',
            markerEnd: { type: MarkerType.ArrowClosed },
          }}
          proOptions={{ hideAttribution: true }}
          className="bg-background"
          style={{ pointerEvents: draggingComponent ? 'none' : 'auto' }}
          connectOnClick={false}
        >
          <Background
            variant={BackgroundVariant.Dots}
            gap={16}
            size={1}
            color="hsl(var(--muted-foreground) / 0.3)"
          />
          <Controls
            showZoom
            showFitView
            showInteractive={false}
            className="bg-background! border-border! shadow-md! [&>button]:bg-background! [&>button]:border-border! [&>button]:text-muted-foreground! [&>button:hover]:bg-accent!"
          />
          <Panel position="top-right" className="mt-2 mr-2">
            <button
              type="button"
              onClick={onClearSteps}
              disabled={clearStepsDisabled}
              className="flex h-8 w-8 items-center justify-center rounded-md border border-border bg-background text-muted-foreground shadow-md transition-colors hover:bg-accent disabled:cursor-not-allowed disabled:opacity-50"
              aria-label={t('editor.menu.clearSteps')}
              title={t('editor.menu.clearSteps')}
            >
              <Trash2 className="h-3.5 w-3.5" />
            </button>
          </Panel>
        </ReactFlow>

        {draggingComponent && (
          <div className="absolute inset-0 z-50 flex items-center justify-center bg-primary/5">
            <div className="px-4 py-2 bg-primary/10 border border-primary/30 rounded-lg pointer-events-none">
              <span className="text-sm font-medium text-primary">{t('editor.clickToAdd')}</span>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

export function FlowCanvas(props: FlowCanvasProps) {
  return (
    <ReactFlowProvider>
      <FlowCanvasInner {...props} />
    </ReactFlowProvider>
  );
}
















