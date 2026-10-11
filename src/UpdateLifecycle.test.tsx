import { act, renderHook, waitFor, cleanup } from '@testing-library/react';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import type { Update } from '@tauri-apps/plugin-updater';
import * as api from './api';
import * as updater from './updater';
import { useDisRouteController } from './useDisRouteController';

vi.mock('./api', () => ({ getStatus: vi.fn(), loadProfile: vi.fn(), saveProfile: vi.fn(), forgetProfile: vi.fn(), getCommunitySnapshot: vi.fn(), checkPrerequisites: vi.fn(), getConnectionHealth: vi.fn(), connect: vi.fn(), connectCommunity: vi.fn(), scanCommunity: vi.fn(), disconnect: vi.fn() }));
vi.mock('./updater', () => ({ findUpdate: vi.fn(), downloadAndInstall: vi.fn() }));
const disconnected = { status: 'disconnected' as const, engineReady: true, isElevated: true, message: '' };
const profile = { name: 'synthetic', configLink: 'trojan://synthetic@example.com:443' };
beforeEach(() => {
  vi.resetAllMocks();
  vi.mocked(api.getStatus).mockResolvedValue(disconnected);
  vi.mocked(api.loadProfile).mockResolvedValue(profile);
  vi.mocked(api.checkPrerequisites).mockResolvedValue(null);
  vi.mocked(api.getConnectionHealth).mockResolvedValue(null);
  vi.mocked(api.disconnect).mockResolvedValue(disconnected);
  vi.mocked(api.getCommunitySnapshot).mockResolvedValue({ sources: [], candidates: [], stale: false, acknowledgedWarning: false, automaticFailover: false });
  vi.mocked(updater.findUpdate).mockResolvedValue({ version: '99.0.0', body: '', close: vi.fn().mockResolvedValue(undefined) } as unknown as Update);
});
afterEach(cleanup);

async function ready() {
  const hook = renderHook(useDisRouteController);
  await waitFor(() => expect(hook.result.current.loading).toBe(false));
  await act(() => hook.result.current.handleCheckUpdates());
  return hook;
}

it('blocks duplicate updates and new connections while preserving the saved configuration', async () => {
  let finish!: () => void;
  vi.mocked(updater.downloadAndInstall).mockImplementation(async () => new Promise<void>(resolve => { finish = resolve; }));
  const { result } = await ready();
  let operation!: Promise<void>;
  act(() => { operation = result.current.handleInstallUpdate(); });
  expect(result.current.connectDisabled).toBe(true);
  await act(async () => {
    await result.current.handleInstallUpdate();
    await result.current.handleSubmit({ preventDefault() {} } as React.FormEvent);
    await result.current.handleCommunityConnect({ preventDefault() {} } as React.FormEvent);
  });
  expect(updater.downloadAndInstall).toHaveBeenCalledOnce();
  expect(api.connect).not.toHaveBeenCalled();
  expect(api.scanCommunity).not.toHaveBeenCalled();
  expect(api.saveProfile).not.toHaveBeenCalled();
  expect(api.forgetProfile).not.toHaveBeenCalled();
  await act(async () => { finish(); await operation; });
  expect(result.current.profile).toEqual(profile);
});

it('reads live connection state immediately before installing rather than a stale render value', async () => {
  let prepare!: () => Promise<void>;
  let finish!: () => void;
  vi.mocked(updater.downloadAndInstall).mockImplementation(async (_update, _report, before) => {
    prepare = before;
    await new Promise<void>(resolve => { finish = resolve; });
  });
  const { result } = await ready();
  let operation!: Promise<void>;
  act(() => { operation = result.current.handleInstallUpdate(); });
  expect(api.disconnect).not.toHaveBeenCalled();
  vi.mocked(api.getStatus).mockResolvedValue({ ...disconnected, status: 'connected' });
  await act(async () => { await prepare(); finish(); await operation; });
  expect(api.disconnect).toHaveBeenCalledOnce();
  expect(api.forgetProfile).not.toHaveBeenCalled();
});

it('keeps the connection and profile after download failure and allows a fresh update check', async () => {
  vi.mocked(updater.downloadAndInstall).mockRejectedValue(new Error('signature verification failed'));
  const { result } = await ready();
  await act(() => result.current.handleInstallUpdate());
  expect(result.current.updatePhase).toBe('error');
  expect(api.disconnect).not.toHaveBeenCalled();
  expect(api.forgetProfile).not.toHaveBeenCalled();
  expect(result.current.profile).toEqual(profile);
  expect(result.current.connectDisabled).toBe(false);
});
