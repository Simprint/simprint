import { memo } from 'react';
import type { ElementType } from 'react';
import { Handle, Position } from '@xyflow/react';
import type { NodeProps } from '@xyflow/react';
import { useTranslation } from 'react-i18next';
import {
  AlertCircle,
  CheckCircle2,
  Clock3,
  Code,
  Database,
  Download,
  FileText,
  GitBranch,
  Globe,
  Keyboard,
  Loader2,
  LogOut,
  Mail,
  MessageSquare,
  MousePointer2,
  Navigation,
  PanelsTopLeft,
  Repeat,
  Upload,
  Variable,
  X,
  Camera,
} from 'lucide-react';

const stepIcons: Record<string, ElementType> = {
  click: MousePointer2,
  input: Keyboard,
  navigate: Navigation,
  select_tab: PanelsTopLeft,
  close_tab: X,
  screenshot: Camera,
  scroll: Globe,
  wait: Clock3,
  condition: GitBranch,
  loop: Repeat,
  break_loop: LogOut,
  extract: FileText,
  download: Download,
  upload: Upload,
  variable: Variable,
  email: Mail,
  api: Database,
  script: Code,
  notify: MessageSquare,
};

const stepColors: Record<string, string> = {
  click: 'border-blue-500/50 bg-blue-500/5',
  input: 'border-purple-500/50 bg-purple-500/5',
  navigate: 'border-cyan-500/50 bg-cyan-500/5',
  select_tab: 'border-blue-600/50 bg-blue-600/5',
  close_tab: 'border-red-500/50 bg-red-500/5',
  screenshot: 'border-pink-500/50 bg-pink-500/5',
  scroll: 'border-teal-500/50 bg-teal-500/5',
  wait: 'border-amber-500/50 bg-amber-500/5',
  condition: 'border-orange-500/50 bg-orange-500/5',
  loop: 'border-green-500/50 bg-green-500/5',
  break_loop: 'border-amber-600/50 bg-amber-600/5',
  extract: 'border-indigo-500/50 bg-indigo-500/5',
  download: 'border-emerald-500/50 bg-emerald-500/5',
  upload: 'border-rose-500/50 bg-rose-500/5',
  variable: 'border-violet-500/50 bg-violet-500/5',
  email: 'border-red-500/50 bg-red-500/5',
  api: 'border-sky-500/50 bg-sky-500/5',
  script: 'border-slate-500/50 bg-slate-500/5',
  notify: 'border-lime-500/50 bg-lime-500/5',
};

const iconColors: Record<string, string> = {
  click: 'text-blue-500',
  input: 'text-purple-500',
  navigate: 'text-cyan-500',
  select_tab: 'text-blue-600',
  close_tab: 'text-red-500',
  screenshot: 'text-pink-500',
  scroll: 'text-teal-500',
  wait: 'text-amber-500',
  condition: 'text-orange-500',
  loop: 'text-green-500',
  break_loop: 'text-amber-600',
  extract: 'text-indigo-500',
  download: 'text-emerald-500',
  upload: 'text-rose-500',
  variable: 'text-violet-500',
  email: 'text-red-500',
  api: 'text-sky-500',
  script: 'text-slate-500',
  notify: 'text-lime-500',
};

export interface StepNodeData {
  label: string;
  subtitle?: string;
  type: string;
  enabled: boolean;
  runStatus?: 'pending' | 'running' | 'awaiting_input' | 'success' | 'failed';
  errorSummary?: string;
  onDelete?: (id: string) => void;
}

function StepNodeComponent({ id, data, selected }: NodeProps) {
  const { t } = useTranslation('rpa');
  const nodeData = data as unknown as StepNodeData;
  const Icon = stepIcons[nodeData.type] || Code;
  const colorClass = stepColors[nodeData.type] || 'border-border bg-background';
  const iconColor = iconColors[nodeData.type] || 'text-muted-foreground';
  const runStatus = nodeData.runStatus ?? 'pending';
  const errorSummary = nodeData.errorSummary?.trim();
  const subtitle = nodeData.subtitle?.trim();
  const hasStatusTag = runStatus !== 'pending';
  const hasMetaRow = Boolean(subtitle) || hasStatusTag;
  const isCondition = nodeData.type === 'condition';
  const hasSourceHandle = nodeData.type !== 'break_loop';

  return (
    <div
      className={`
        w-[210px] px-3 py-2 rounded-lg border-2 shadow-sm transition-all relative
        ${colorClass}
        ${selected ? 'ring-2 ring-primary ring-offset-1 ring-offset-background' : ''}
        ${!nodeData.enabled ? 'opacity-50' : ''}
        ${runStatus === 'running' ? 'ring-2 ring-sky-500 ring-offset-1 ring-offset-background' : ''}
        ${runStatus === 'awaiting_input' ? 'ring-2 ring-amber-500 ring-offset-1 ring-offset-background' : ''}
        ${runStatus === 'success' ? 'border-emerald-500/60 bg-emerald-500/10' : ''}
        ${runStatus === 'failed' ? 'border-destructive/70 bg-destructive/10' : ''}
      `}
    >
      <Handle
        type="target"
        position={Position.Top}
        className="!w-3 !h-3 !bg-muted-foreground !border-2 !border-background"
      />

      <div className="flex items-center gap-2">
        <div className={`w-7 h-7 rounded flex items-center justify-center bg-background/50 ${iconColor}`}>
          <Icon className="h-4 w-4" />
        </div>
        <div className={`flex-1 min-w-0 ${hasMetaRow ? '' : 'flex items-center min-h-7'}`}>
          <div className="text-xs font-medium text-foreground truncate max-w-[168px]">{nodeData.label}</div>
          {hasMetaRow ? (
            <div className="text-[10px] text-muted-foreground flex items-center gap-1.5 min-h-4 min-w-0">
              {subtitle ? <span className="truncate flex-1 min-w-0">{subtitle}</span> : null}
              {runStatus === 'running' && (
                <span className="inline-flex shrink-0 items-center gap-1 whitespace-nowrap text-sky-500">
                  <Loader2 className="h-3 w-3 animate-spin" />
                  <span>{t('editor.canvas.status.running')}</span>
                </span>
              )}
              {runStatus === 'awaiting_input' && (
                <span className="inline-flex shrink-0 items-center gap-1 whitespace-nowrap text-amber-600">
                  <Loader2 className="h-3 w-3 animate-spin" />
                  <span>{t('editor.canvas.status.awaitingInput')}</span>
                </span>
              )}
              {runStatus === 'success' && (
                <span className="inline-flex shrink-0 items-center gap-1 whitespace-nowrap text-emerald-500">
                  <CheckCircle2 className="h-3 w-3" />
                  <span>{t('editor.canvas.status.success')}</span>
                </span>
              )}
              {runStatus === 'failed' && (
                <span className="inline-flex shrink-0 items-center gap-1 whitespace-nowrap text-destructive">
                  <AlertCircle className="h-3 w-3" />
                  <span>{t('editor.canvas.status.failed')}</span>
                </span>
              )}
            </div>
          ) : null}
          {runStatus === 'failed' && errorSummary ? (
            <div className="mt-1 text-[10px] text-destructive leading-4 break-words max-w-[180px]">
              {errorSummary}
            </div>
          ) : null}
        </div>
        {nodeData.onDelete && (
          <button
            onClick={(e) => {
              e.stopPropagation();
              nodeData.onDelete?.(id);
            }}
            className="h-5 w-5 flex items-center justify-center rounded hover:bg-destructive/20 text-muted-foreground hover:text-destructive transition-colors"
          >
            <X className="h-3 w-3" />
          </button>
        )}
      </div>

      {isCondition ? (
        <>
          <Handle
            id="branch-true"
            type="source"
            position={Position.Bottom}
            style={{ left: '28%' }}
            className="!w-3 !h-3 !bg-emerald-500 !border-2 !border-background"
          />
          <Handle
            id="branch-false"
            type="source"
            position={Position.Bottom}
            style={{ left: '72%' }}
            className="!w-3 !h-3 !bg-rose-500 !border-2 !border-background"
          />
        </>
      ) : hasSourceHandle ? (
        <Handle
          type="source"
          position={Position.Bottom}
          className="!w-3 !h-3 !bg-muted-foreground !border-2 !border-background"
        />
      ) : null}
    </div>
  );
}

export const StepNode = memo(StepNodeComponent);

