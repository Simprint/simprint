import type { FlowStep } from './flow-canvas';

export const LOOP_REGION_WIDTH = 380;
export const LOOP_REGION_HEIGHT = 320;
export const LOOP_REGION_PADDING_X = 20;
export const LOOP_REGION_PADDING_RIGHT = 20;
export const LOOP_REGION_HEADER_HEIGHT = 52;
export const LOOP_REGION_HINT_HEIGHT = 44;
export const LOOP_REGION_HINT_GAP = 12;
export const LOOP_REGION_CONTENT_TOP = LOOP_REGION_HEADER_HEIGHT + 12;
export const LOOP_REGION_CONTENT_BOTTOM = 20;
export const LOOP_REGION_GAP_X = 16;
export const LOOP_REGION_GAP_Y = 16;
export const DEFAULT_STEP_WIDTH = 170;
export const DEFAULT_STEP_HEIGHT = 72;

export function getLoopDimensions(step: FlowStep): { width: number; height: number } {
  const width =
    typeof step.config.loopWidth === 'number' && step.config.loopWidth >= 280
      ? step.config.loopWidth
      : LOOP_REGION_WIDTH;
  const height =
    typeof step.config.loopHeight === 'number' && step.config.loopHeight >= 220
      ? step.config.loopHeight
      : LOOP_REGION_HEIGHT;

  return { width, height };
}

function getLoopColumnCount(width: number): number {
  return Math.max(
    1,
    Math.floor((width - LOOP_REGION_PADDING_X - LOOP_REGION_PADDING_RIGHT + LOOP_REGION_GAP_X) / (DEFAULT_STEP_WIDTH + LOOP_REGION_GAP_X))
  );
}

export function getLoopAutoDimensions(step: FlowStep, childCount: number): { width: number; height: number } {
  const current = getLoopDimensions(step);
  const width = Math.max(current.width, LOOP_REGION_WIDTH);
  const columns = getLoopColumnCount(width);
  const rows = Math.max(1, Math.ceil(Math.max(childCount, 1) / columns));
  const requiredHeight =
    LOOP_REGION_CONTENT_TOP +
    rows * DEFAULT_STEP_HEIGHT +
    Math.max(0, rows - 1) * LOOP_REGION_GAP_Y +
    LOOP_REGION_CONTENT_BOTTOM;

  return {
    width,
    height: Math.max(current.height, requiredHeight),
  };
}

export function getLoopAutoPosition(step: FlowStep, childIndex: number): { x: number; y: number } {
  const { width } = getLoopAutoDimensions(step, childIndex + 1);
  const columns = getLoopColumnCount(width);
  const column = childIndex % columns;
  const row = Math.floor(childIndex / columns);

  return {
    x: LOOP_REGION_PADDING_X + column * (DEFAULT_STEP_WIDTH + LOOP_REGION_GAP_X),
    y: LOOP_REGION_CONTENT_TOP + row * (DEFAULT_STEP_HEIGHT + LOOP_REGION_GAP_Y),
  };
}

export function getLoopExpandedWidthForChild(step: FlowStep, childX: number): number {
  const current = getLoopDimensions(step);
  const requiredWidth = childX + DEFAULT_STEP_WIDTH + LOOP_REGION_PADDING_RIGHT;
  return Math.max(current.width, requiredWidth);
}

export function getLoopExpandedHeightForChild(step: FlowStep, childY: number): number {
  const current = getLoopDimensions(step);
  const requiredHeight = childY + DEFAULT_STEP_HEIGHT + LOOP_REGION_CONTENT_BOTTOM;
  return Math.max(current.height, requiredHeight);
}

export function getLoopWidthFromChildren(step: FlowStep, children: FlowStep[]): number {
  if (children.length === 0) {
    return LOOP_REGION_WIDTH;
  }

  const maxRight = children.reduce((current, child) => {
    const childX = child.position?.x ?? LOOP_REGION_PADDING_X;
    return Math.max(current, childX + DEFAULT_STEP_WIDTH + LOOP_REGION_PADDING_RIGHT);
  }, LOOP_REGION_WIDTH);

  return Math.max(LOOP_REGION_WIDTH, maxRight);
}

export function getLoopHeightFromChildren(step: FlowStep, children: FlowStep[]): number {
  if (children.length === 0) {
    return LOOP_REGION_HEIGHT;
  }

  const maxBottom = children.reduce((current, child) => {
    const childY = child.position?.y ?? LOOP_REGION_CONTENT_TOP;
    return Math.max(current, childY + DEFAULT_STEP_HEIGHT + LOOP_REGION_CONTENT_BOTTOM);
  }, LOOP_REGION_HEIGHT);

  return Math.max(LOOP_REGION_HEIGHT, maxBottom);
}



