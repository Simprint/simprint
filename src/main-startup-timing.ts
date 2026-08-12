export interface MainStartupTiming {
  htmlStartedAt: number;
  moduleExecutedAt?: number;
  reactRenderScheduledAt?: number;
  windowManagerEffectStartedAt?: number;
  firstContentLayoutAt?: number;
  fontsReadyAt?: number;
  readyInvokeAt?: number;
}

type MainTimingWindow = Window & {
  __SIMPRINT_MAIN_HTML_STARTED_AT__?: number;
};

const timingWindow = window as MainTimingWindow;

export const mainStartupTiming: MainStartupTiming = {
  // The inline HTML marker runs before the module graph is fetched and evaluated.
  htmlStartedAt: timingWindow.__SIMPRINT_MAIN_HTML_STARTED_AT__ ?? performance.now(),
};

export function elapsedSinceMainHtml(timestamp: number | undefined): number | undefined {
  return timestamp === undefined ? undefined : timestamp - mainStartupTiming.htmlStartedAt;
}
