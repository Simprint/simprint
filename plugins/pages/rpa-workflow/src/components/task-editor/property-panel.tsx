import { open as openFileDialog } from '@tauri-apps/plugin-dialog';
import { useTranslation } from 'react-i18next';
import { Settings, Trash2 } from 'lucide-react';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Textarea } from '@/components/ui/textarea';
import { Switch } from '@/components/ui/switch';
import { Button } from '@/components/ui/button';
import { ScrollArea } from '@/components/ui/scroll-area';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { NumberStepInput } from '../number-step-input';
import type { FlowStep } from './flow-canvas';

interface PropertyPanelProps {
  step: FlowStep | null;
  onUpdateStep: (step: FlowStep) => void;
  onDeleteStep: (id: string) => void;
  embedded?: boolean;
}

export function PropertyPanel({ step, onUpdateStep, onDeleteStep, embedded = false }: PropertyPanelProps) {
  const { t } = useTranslation('rpa');

  if (!step) {
    return (
      <div className={embedded ? 'bg-background flex flex-col min-h-0 overflow-hidden h-full' : 'w-72 border-l border-border bg-background flex flex-col min-h-0 overflow-hidden'}>
        {!embedded ? (
          <div className="px-3 py-2 border-b border-border shrink-0">
            <h3 className="text-xs font-semibold text-foreground">{t('editor.properties')}</h3>
          </div>
        ) : null}
        <div className="flex-1 flex items-center justify-center p-4 min-h-0">
          <div className="text-center">
            <Settings className="h-8 w-8 text-muted-foreground/30 mx-auto mb-2" />
            <p className="text-xs text-muted-foreground">{t('editor.selectStepToEdit')}</p>
          </div>
        </div>
      </div>
    );
  }

  const handleConfigChange = (key: string, value: unknown) => {
    onUpdateStep({
      ...step,
      config: {
        ...step.config,
        [key]: value,
      },
    });
  };

  const hasRecordedSelector = typeof step.config.selector === 'string' && step.config.selector.trim();

  const clearRecordedTarget = () => {
    onUpdateStep({
      ...step,
      config: {
        ...step.config,
        selector: '',
        selectorCandidates: [],
        selectorSnapshot: undefined,
      },
    });
  };

  const renderRecordedTargetAction = () => {
    if (!hasRecordedSelector) {
      return null;
    }

    return (
      <div className="flex items-center justify-between rounded-md border border-border px-3 py-2">
        <span className="text-xs text-muted-foreground">
          {t('editor.condition.targetRecorded', { defaultValue: '已记录目标' })}
        </span>
        <Button
          type="button"
          variant="outline"
          size="sm"
          className="h-7 px-2 text-[10px]"
          onClick={clearRecordedTarget}
        >
          {t('editor.condition.clearTarget', { defaultValue: '清除目标' })}
        </Button>
      </div>
    );
  };

  const handlePickUploadFile = async () => {
    const selected = await openFileDialog({
      multiple: false,
      directory: false,
    });

    if (typeof selected !== 'string' || !selected.trim()) {
      return;
    }

    onUpdateStep({
      ...step,
      config: {
        ...step.config,
        filePath: selected,
      },
    });
  };

  const renderStepConfig = () => {
    switch (step.type) {
      case 'click':
        return (
          <>
            <div className="space-y-2">
              <Label className="text-xs">{t('editor.config.selector')}</Label>
              <Input
                value={(step.config.selector as string) || ''}
                onChange={(e) => handleConfigChange('selector', e.target.value)}
                placeholder="#button, .class, xpath..."
                className="h-8 text-xs"
              />
              {renderRecordedTargetAction()}
            </div>
            <div className="space-y-2">
              <Label className="text-xs">{t('editor.config.clickType')}</Label>
              <Select
                value={(step.config.clickType as string) || 'single'}
                onValueChange={(v) => handleConfigChange('clickType', v)}
              >
                <SelectTrigger className="h-8 text-xs w-full">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="single">{t('editor.config.singleClick')}</SelectItem>
                  <SelectItem value="double">{t('editor.config.doubleClick')}</SelectItem>
                  <SelectItem value="right">{t('editor.config.rightClick')}</SelectItem>
                </SelectContent>
              </Select>
            </div>
            <div className="flex items-center justify-between">
              <Label className="text-xs">{t('editor.config.waitForElement')}</Label>
              <Switch
                checked={(step.config.waitForElement as boolean) ?? true}
                onCheckedChange={(v) => handleConfigChange('waitForElement', v)}
              />
            </div>
          </>
        );

      case 'input':
        return (
          <>
            <div className="space-y-2">
              <Label className="text-xs">{t('editor.config.selector')}</Label>
              <Input
                value={(step.config.selector as string) || ''}
                onChange={(e) => handleConfigChange('selector', e.target.value)}
                placeholder="#input, .class, xpath..."
                className="h-8 text-xs"
              />
              {renderRecordedTargetAction()}
            </div>
            <div className="space-y-2">
              <Label className="text-xs">{t('editor.config.inputText')}</Label>
              <Textarea
                value={(step.config.text as string) || ''}
                onChange={(e) => handleConfigChange('text', e.target.value)}
                placeholder={t('editor.config.inputTextPlaceholder')}
                className="text-xs min-h-[60px] resize-none"
              />
            </div>
            <div className="flex items-center justify-between">
              <Label className="text-xs">{t('editor.config.clearFirst')}</Label>
              <Switch
                checked={(step.config.clearFirst as boolean) ?? true}
                onCheckedChange={(v) => handleConfigChange('clearFirst', v)}
              />
            </div>
            <div className="flex items-center justify-between">
              <Label className="text-xs">{t('editor.config.typeSlowly')}</Label>
              <Switch
                checked={(step.config.typeSlowly as boolean) ?? false}
                onCheckedChange={(v) => handleConfigChange('typeSlowly', v)}
              />
            </div>
          </>
        );

      case 'screenshot':
        return (
          <>
            <div className="space-y-2">
              <Label className="text-xs">{t('editor.config.captureMode', { defaultValue: '截图模式' })}</Label>
              <Select
                value={(step.config.captureMode as string) || 'viewport'}
                onValueChange={(v) => handleConfigChange('captureMode', v)}
              >
                <SelectTrigger className="h-8 text-xs w-full">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="viewport">{t('editor.config.viewportScreenshot', { defaultValue: '视口截图' })}</SelectItem>
                  <SelectItem value="full_page">{t('editor.config.fullPageScreenshot', { defaultValue: '全页截图' })}</SelectItem>
                  <SelectItem value="element">{t('editor.config.elementScreenshot', { defaultValue: '区域截图' })}</SelectItem>
                </SelectContent>
              </Select>
            </div>
            {(step.config.captureMode as string) === 'element' && (
              <div className="space-y-2">
                <div className="rounded-md border border-dashed border-border px-3 py-3 text-xs text-muted-foreground">
                  {t('editor.config.elementScreenshotHint', { defaultValue: '运行时在打开的浏览器中点击目标区域，系统会自动记录并提交。' })}
                </div>
                {renderRecordedTargetAction()}
              </div>
            )}
            <div className="space-y-2">
              <Label className="text-xs">{t('editor.config.screenshotPreview', { defaultValue: '截图预览' })}</Label>
              {typeof step.config.lastScreenshot === 'string' && step.config.lastScreenshot ? (
                <div className="rounded-md border border-border overflow-hidden bg-muted/20">
                  <img
                    src={step.config.lastScreenshot as string}
                    alt={t('editor.config.screenshotPreview', { defaultValue: '截图预览' })}
                    className="block w-full h-auto"
                  />
                </div>
              ) : (
                <div className="rounded-md border border-dashed border-border px-3 py-6 text-center text-xs text-muted-foreground">
                  {t('editor.config.noScreenshotYet', { defaultValue: '运行后将在这里显示截图结果' })}
                </div>
              )}
            </div>
          </>
        );

      case 'scroll':
        return (
          <>
            <div className="space-y-2">
              <Label className="text-xs">{t('editor.config.scrollMode', { defaultValue: '滚动模式' })}</Label>
              <Select
                value={(step.config.scrollMode as string) || 'to_element'}
                onValueChange={(v) => handleConfigChange('scrollMode', v)}
              >
                <SelectTrigger className="h-8 text-xs w-full">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="to_element">{t('editor.config.scrollToElement', { defaultValue: '滚动到元素' })}</SelectItem>
                  <SelectItem value="by_viewport">{t('editor.config.scrollByViewport', { defaultValue: '按屏滚动' })}</SelectItem>
                  <SelectItem value="to_edge">{t('editor.config.scrollToEdge', { defaultValue: '滚动到边缘' })}</SelectItem>
                </SelectContent>
              </Select>
            </div>
            {((step.config.scrollMode as string) || 'to_element') === 'to_element' && (
              <div className="space-y-2">
                <div className="rounded-md border border-dashed border-border px-3 py-3 text-xs text-muted-foreground">
                  {t('editor.config.scrollElementHint', { defaultValue: '运行时在打开的浏览器中点击目标位置，系统会自动记录并提交。' })}
                </div>
                {renderRecordedTargetAction()}
              </div>
            )}
            {((step.config.scrollMode as string) || 'to_element') === 'by_viewport' && (
              <>
                <div className="space-y-2">
                  <Label className="text-xs">{t('editor.config.scrollDirection', { defaultValue: '滚动方向' })}</Label>
                  <Select
                    value={(step.config.direction as string) || 'down'}
                    onValueChange={(v) => handleConfigChange('direction', v)}
                  >
                    <SelectTrigger className="h-8 text-xs w-full">
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="down">{t('editor.config.scrollDown', { defaultValue: '向下' })}</SelectItem>
                      <SelectItem value="up">{t('editor.config.scrollUp', { defaultValue: '向上' })}</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
                <div className="space-y-2">
                  <Label className="text-xs">{t('editor.config.viewportCount', { defaultValue: '滚动屏数' })}</Label>
                  <NumberStepInput
                    value={(step.config.viewportCount as number) || 1}
                    min={1}
                    onChange={(value) => handleConfigChange('viewportCount', value)}
                    inputClassName="w-12 text-xs"
                    decreaseLabel="减少滚动屏数"
                    increaseLabel="增加滚动屏数"
                  />
                </div>
              </>
            )}
            {((step.config.scrollMode as string) || 'to_element') === 'to_edge' && (
              <div className="space-y-2">
                <Label className="text-xs">{t('editor.config.scrollEdge', { defaultValue: '边缘位置' })}</Label>
                <Select
                  value={(step.config.edge as string) || 'bottom'}
                  onValueChange={(v) => handleConfigChange('edge', v)}
                >
                  <SelectTrigger className="h-8 text-xs w-full">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="top">{t('editor.config.scrollTop', { defaultValue: '顶部' })}</SelectItem>
                    <SelectItem value="bottom">{t('editor.config.scrollBottom', { defaultValue: '底部' })}</SelectItem>
                  </SelectContent>
                </Select>
              </div>
            )}
          </>
        );

      case 'navigate':
        return (
          <>
            <div className="space-y-2">
              <Label className="text-xs">{t('editor.config.url')}</Label>
              <Input
                value={(step.config.url as string) || ''}
                onChange={(e) => handleConfigChange('url', e.target.value)}
                placeholder="https://example.com"
                className="h-8 text-xs"
              />
            </div>
            <div className="flex items-center justify-between">
              <Label className="text-xs">{t('editor.config.waitForLoad')}</Label>
              <Switch
                checked={(step.config.waitForLoad as boolean) ?? true}
                onCheckedChange={(v) => handleConfigChange('waitForLoad', v)}
              />
            </div>
          </>
        );

      case 'select_tab':
        return (
          <div className="space-y-2">
            <Label className="text-xs">
              {t('editor.config.tabIndex', { defaultValue: '标签页位置' })}
            </Label>
            <NumberStepInput
              value={(step.config.tabIndex as number) || 1}
              min={1}
              onChange={(value) => handleConfigChange('tabIndex', Math.max(1, Math.trunc(value)))}
              className="w-full"
              inputClassName="w-full px-3 text-left text-xs"
              decreaseLabel="减少标签页位置"
              increaseLabel="增加标签页位置"
            />
          </div>
        );

      case 'close_tab':
        return (
          <div className="space-y-2">
            <Label className="text-xs">
              {t('editor.config.tabIndex', { defaultValue: '标签页位置' })}
            </Label>
            <NumberStepInput
              value={(step.config.tabIndex as number) || 1}
              min={1}
              onChange={(value) => handleConfigChange('tabIndex', Math.max(1, Math.trunc(value)))}
              className="w-full"
              inputClassName="w-full px-3 text-left text-xs"
              decreaseLabel="减少标签页位置"
              increaseLabel="增加标签页位置"
            />
          </div>
        );

      case 'wait':
        return (
          <>
            <div className="space-y-2">
              <Label className="text-xs">{t('editor.config.waitType', { defaultValue: '等待类型' })}</Label>
              <Select
                value={(step.config.waitType as string) || 'time'}
                onValueChange={(v) => handleConfigChange('waitType', v)}
              >
                <SelectTrigger className="h-8 text-xs w-full">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="time">{t('editor.config.waitTime', { defaultValue: '等待时间' })}</SelectItem>
                  <SelectItem value="element">{t('editor.config.waitElement', { defaultValue: '等待元素' })}</SelectItem>
                </SelectContent>
              </Select>
            </div>
            {((step.config.waitType as string) || 'time') === 'time' && (
              <div className="space-y-2">
                <Label className="text-xs">{t('editor.config.duration', { defaultValue: '等待时长' })}</Label>
                <div className="flex items-center gap-2">
                  <NumberStepInput
                    value={(step.config.duration as number) || 1000}
                    min={0}
                    onChange={(value) => handleConfigChange('duration', value)}
                    className="w-full"
                    inputClassName="w-full px-3 text-left text-xs"
                    decreaseLabel="减少等待时长"
                    increaseLabel="增加等待时长"
                  />
                  <span className="text-xs text-muted-foreground">ms</span>
                </div>
              </div>
            )}
            {((step.config.waitType as string) || 'time') === 'element' && (
              <div className="space-y-2">
                <div className="rounded-md border border-dashed border-border px-3 py-3 text-xs text-muted-foreground">
                  {t('editor.config.waitElementHint', { defaultValue: '运行时在打开的浏览器中点击等待目标，系统会自动记录并提交。' })}
                </div>
                {renderRecordedTargetAction()}
              </div>
            )}
          </>
        );

      case 'condition':
        return (
          <>
            <div className="space-y-2">
              <Label className="text-xs">{t('editor.condition.typeLabel', { defaultValue: '判断类型' })}</Label>
              <Select
                value={(step.config.conditionType as string) || 'element_visible'}
                onValueChange={(v) => handleConfigChange('conditionType', v)}
              >
                <SelectTrigger className="h-8 text-xs w-full">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="element_visible">{t('editor.condition.elementVisible', { defaultValue: '页面元素可见' })}</SelectItem>
                  <SelectItem value="text_present">{t('editor.condition.textPresent', { defaultValue: '页面文本出现' })}</SelectItem>
                  <SelectItem value="url_contains">{t('editor.condition.urlContains', { defaultValue: '页面地址包含' })}</SelectItem>
                </SelectContent>
              </Select>
            </div>
            {((step.config.conditionType as string) || 'element_visible') === 'element_visible' && (
              <div className="space-y-2">
                <div className="rounded-md border border-dashed border-border px-3 py-3 text-xs text-muted-foreground">
                  {t('editor.condition.elementHint', { defaultValue: '运行时在打开的浏览器中点击判断目标，系统会自动记录并提交。' })}
                </div>
                {renderRecordedTargetAction()}
              </div>
            )}
            {((step.config.conditionType as string) || 'element_visible') === 'text_present' && (
              <div className="space-y-2">
                <Label className="text-xs">{t('editor.condition.textValue', { defaultValue: '目标文本' })}</Label>
                <Input
                  value={(step.config.expectedText as string) || ''}
                  onChange={(e) => handleConfigChange('expectedText', e.target.value)}
                  placeholder={t('editor.condition.textPlaceholder', { defaultValue: '请输入目标文本' })}
                  className="h-8 text-xs"
                />
              </div>
            )}
            {((step.config.conditionType as string) || 'element_visible') === 'url_contains' && (
              <div className="space-y-2">
                <Label className="text-xs">{t('editor.condition.urlValue', { defaultValue: '地址片段' })}</Label>
                <Input
                  value={(step.config.urlFragment as string) || ''}
                  onChange={(e) => handleConfigChange('urlFragment', e.target.value)}
                  placeholder={t('editor.condition.urlPlaceholder', { defaultValue: '请输入地址片段' })}
                  className="h-8 text-xs"
                />
              </div>
            )}
            <div className="rounded-md border border-border/60 bg-muted/20 px-3 py-3 text-xs text-muted-foreground">
              {t('editor.condition.branchHint', { defaultValue: '条件节点支持“是/否”两个子节点，请在画布上分别连接分支。' })}
            </div>
          </>
        );

      case 'loop':
        return (
          <>
            <div className="space-y-2">
              <Label className="text-xs">{t('editor.config.iterations', { defaultValue: '循环次数' })}</Label>
              <NumberStepInput
                value={(step.config.iterations as number) || 3}
                min={1}
                onChange={(value) => handleConfigChange('iterations', Math.max(1, value))}
                className="w-full"
                inputClassName="w-full px-3 text-left text-xs"
                decreaseLabel="减少循环次数"
                increaseLabel="增加循环次数"
              />
            </div>
            <div className="rounded-md border border-border/60 bg-muted/20 px-3 py-3 text-xs text-muted-foreground">
              {t('editor.loop.regionHint', { defaultValue: '循环节点是一个区域容器。将需要重复执行的节点拖入该区域，区域外连线表示循环完成后继续执行。' })}
            </div>
          </>
        );

      case 'break_loop':
        return (
          <div className="rounded-md border border-border/60 bg-muted/20 px-3 py-3 text-xs text-muted-foreground">
            {t('editor.stepSubtitleDefaults.breakLoop', {
              defaultValue: '满足条件时立即退出当前循环',
            })}
          </div>
        );

      case 'extract':
        return (
          <>
            <div className="space-y-2">
              <Label className="text-xs">提取类型</Label>
              <Select
                value={(step.config.extractType as string) || 'text'}
                onValueChange={(v) => handleConfigChange('extractType', v)}
              >
                <SelectTrigger className="h-8 text-xs w-full">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="text">文本</SelectItem>
                  <SelectItem value="href">链接</SelectItem>
                  <SelectItem value="value">输入值</SelectItem>
                </SelectContent>
              </Select>
            </div>

            <div className="space-y-2">
              <Label className="text-xs">变量名</Label>
              <Input
                value={(step.config.outputKey as string) || ''}
                onChange={(e) => handleConfigChange('outputKey', e.target.value)}
                placeholder="例如：extract_ab12cd"
                className="h-8 text-xs"
              />
            </div>

            <div className="space-y-2">
              <div className="rounded-md border border-dashed border-border px-3 py-3 text-xs text-muted-foreground">
                运行时在打开的浏览器中点击提取目标，系统会自动记录并预览提取结果。
              </div>
              {renderRecordedTargetAction()}
            </div>

            <div className="space-y-2">
              <Label className="text-xs">提取预览</Label>
              <div className="rounded-md border border-border bg-muted/20 px-3 py-3 text-xs text-muted-foreground break-all">
                {(step.config.previewValue as string) || '运行后将在这里显示提取结果'}
              </div>
            </div>
          </>
        );

      case 'upload':
        return (
          <>
            <div className="space-y-2">
              <Label className="text-xs">本地文件</Label>
              <div className="flex gap-2">
                <Input
                  value={(step.config.filePath as string) || ''}
                  readOnly
                  placeholder="请选择要上传的本地文件"
                  className="h-8 text-xs"
                />
                <Button
                  type="button"
                  variant="outline"
                  size="sm"
                  className="h-8 px-3 text-xs"
                  onClick={() => void handlePickUploadFile()}
                >
                  选择文件
                </Button>
              </div>
            </div>

            <div className="space-y-2">
              <div className="rounded-md border border-dashed border-border px-3 py-3 text-xs text-muted-foreground">
                运行时在打开的浏览器中点击上传控件，系统会自动记录目标并提交文件。
              </div>
              {renderRecordedTargetAction()}
            </div>
          </>
        );

      case 'script':
        return (
          <div className="space-y-2">
            <Label className="text-xs">{t('editor.config.script')}</Label>
            <Textarea
              value={(step.config.script as string) || ''}
              onChange={(e) => handleConfigChange('script', e.target.value)}
              placeholder="return document.title;"
              className="text-xs min-h-[120px] w-full font-mono resize-none break-all whitespace-pre-wrap"
            />
            <div className="space-y-2">
              <Label className="text-xs">变量名</Label>
              <Input
                value={(step.config.outputKey as string) || ''}
                onChange={(e) => handleConfigChange('outputKey', e.target.value)}
                placeholder="例如：script_ab12cd"
                className="h-8 text-xs"
              />
            </div>
            <div className="space-y-2">
              <Label className="text-xs">执行结果预览</Label>
              <div className="rounded-md border border-border bg-muted/20 px-3 py-3 text-xs text-muted-foreground break-all">
                {(step.config.previewValue as string) || '运行后将在这里显示脚本执行结果'}
              </div>
            </div>
          </div>
        );

      default:
        return (
          <div className="text-xs text-muted-foreground text-center py-4">
            {t('editor.configNotAvailable')}
          </div>
        );
    }
  };

  return (
    <div className={embedded ? 'bg-background flex flex-col min-h-0 overflow-hidden h-full' : 'w-72 border-l border-border bg-background flex flex-col min-h-0 overflow-hidden'}>
      {!embedded ? (
        <div className="px-3 py-2 border-b border-border flex items-center justify-between shrink-0">
          <h3 className="text-xs font-semibold text-foreground">{t('editor.properties')}</h3>
          <Button
            variant="ghost"
            size="icon"
            className="h-6 w-6 text-muted-foreground hover:text-destructive hover:bg-destructive/10"
            onClick={() => onDeleteStep(step.id)}
          >
            <Trash2 className="h-3.5 w-3.5" />
          </Button>
        </div>
      ) : null}

      <ScrollArea className="flex-1 min-h-0 overflow-y-auto">
        <div className="p-3 space-y-4">
          {step ? (
            <>
              <div className="space-y-2">
                <Label className="text-xs">{t('editor.config.stepName')}</Label>
                <Input
                  value={step.name}
                  onChange={(e) => onUpdateStep({ ...step, name: e.target.value })}
                  className="h-8 text-xs"
                />
              </div>

              <div className="flex items-center justify-between">
                <Label className="text-xs">{t('editor.config.enabled')}</Label>
                <Switch
                  checked={step.enabled}
                  onCheckedChange={(v) => onUpdateStep({ ...step, enabled: v })}
                />
              </div>
            </>
          ) : null}

          <div className="space-y-2">
            <Label className="text-xs">{t('editor.stepConfig')}</Label>
            <div className="space-y-3 rounded-md border border-border bg-muted/10 p-3">
              {renderStepConfig()}
            </div>
          </div>
        </div>
      </ScrollArea>
    </div>
  );
}

