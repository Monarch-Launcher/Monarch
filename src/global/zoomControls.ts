import { invoke } from '@tauri-apps/api/core';

const ZOOM_STEP = 0.1;
const ZOOM_MIN = 0.5;
const ZOOM_MAX = 3.0;
const STORAGE_KEY = 'uiScale';

let currentScale = 1.0;

const clamp = (val: number, min: number, max: number) => Math.min(Math.max(val, min), max);

function getSavedScale(): number {
  const saved = Number.parseFloat(localStorage.getItem(STORAGE_KEY) || '1');
  if (Number.isFinite(saved)) return clamp(saved, ZOOM_MIN, ZOOM_MAX);
  return 1.0;
}

async function applyScale(scale: number) {
  const clamped = clamp(Number.isFinite(scale) ? scale : 1.0, ZOOM_MIN, ZOOM_MAX);
  currentScale = Math.round(clamped * 100) / 100; // keep two decimals
  localStorage.setItem(STORAGE_KEY, currentScale.toString());
  try {
    await invoke('zoom_window', { scaleFactor: currentScale });
  } catch (e) {
    // eslint-disable-next-line no-console
    console.error('Failed to invoke zoom_window:', e);
  }
}

function onKeyDown(e: KeyboardEvent) {
  const isAccel = e.ctrlKey || e.metaKey;
  if (!isAccel) return;

  const { key } = e;
  if (key === '+' || key === '=') {
    e.preventDefault();
    applyScale(currentScale + ZOOM_STEP).catch(() => {});
  } else if (key === '-') {
    e.preventDefault();
    applyScale(currentScale - ZOOM_STEP).catch(() => {});
  } else if (key === '0') {
    e.preventDefault();
    applyScale(1.0).catch(() => {});
  }
}

export function initZoomControls(): void {
  // Apply saved zoom on startup
  currentScale = getSavedScale();
  applyScale(currentScale).catch(() => {});

  // Register global key handler
  window.addEventListener('keydown', onKeyDown);
}

export function disposeZoomControls(): void {
  window.removeEventListener('keydown', onKeyDown);
}
