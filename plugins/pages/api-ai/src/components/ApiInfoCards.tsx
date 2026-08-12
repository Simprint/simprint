import { ExternalLink } from 'lucide-react';
import { useTranslation } from 'react-i18next';

export function ApiInfoCards() {
  const { t } = useTranslation('apiConsole');

  return (
    <div className="space-y-4">
      {/* API接口频率限制 */}
      <div className="bg-background border border-border p-3">
        <h3 className="text-xs font-semibold text-foreground mb-2">{t('cards.rateLimit')}</h3>
        <p className="text-xs text-muted-foreground leading-relaxed mb-2">
          {t('cards.rateLimitDesc')}
        </p>
      </div>

      {/* API文档 */}
      <div className="bg-background border border-border p-3">
        <h3 className="text-xs font-semibold text-foreground mb-2">{t('cards.apiDocs')}</h3>
        <p className="text-xs text-muted-foreground leading-relaxed mb-3">
          {t('cards.apiDocsDesc')}
        </p>
        <button
          onClick={(e) => {
            e.preventDefault();
            // TODO: 打开API文档
          }}
          className="w-full bg-primary text-primary-foreground text-xs font-semibold px-3 py-1.5 hover:bg-primary/90 transition-colors rounded inline-flex items-center justify-center gap-1.5"
        >
          <ExternalLink className="w-3 h-3" />
          {t('cards.viewFullDocs')}
        </button>
      </div>
    </div>
  );
}
