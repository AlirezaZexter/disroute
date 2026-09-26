import "@testing-library/jest-dom/vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import * as api from "./api";
import App from "./App";

vi.mock('./api', () => ({ getStatus: vi.fn(), loadProfile: vi.fn(), saveProfile: vi.fn(), forgetProfile: vi.fn(), connect: vi.fn(), disconnect: vi.fn(), hideToTray: vi.fn().mockResolvedValue(undefined), restartDiscord: vi.fn(), getCommunitySnapshot: vi.fn(), saveCommunitySources: vi.fn(), setCommunityPreferences: vi.fn(), refreshCommunity: vi.fn(), scanCommunity: vi.fn(), cancelCommunityScan: vi.fn(), clearCommunityData: vi.fn(), connectCommunity: vi.fn() }));
const profile = { name: 'My route', configLink: 'vless://00000000-0000-4000-8000-000000000000@example.com:443?security=tls' };
const status = { status: 'disconnected' as const, engineReady: true, isElevated: true, message: 'ready' };
beforeEach(() => {
  vi.resetAllMocks();
  vi.mocked(api.getStatus).mockResolvedValue(status);
  vi.mocked(api.loadProfile).mockResolvedValue(profile);
  vi.mocked(api.saveProfile).mockResolvedValue();
  vi.mocked(api.forgetProfile).mockResolvedValue();
  vi.mocked(api.connect).mockResolvedValue({ ...status, status: 'connected' });
  vi.mocked(api.restartDiscord).mockResolvedValue('Discord دوباره اجرا شد.');
  vi.mocked(api.getCommunitySnapshot).mockResolvedValue({ sources: [], candidates: [], stale: false, acknowledgedWarning: false, automaticFailover: true });
});
afterEach(cleanup);
it.each([false, true])('uses fresh cache and only refreshes stale cache (stale=%s)', async (stale) => {
  vi.mocked(api.loadProfile).mockResolvedValue(null);
  const snapshot = { sources: [{ id: 'test', name: 'test source', location: 'https://example.org/sub', attribution: 'test', kind: 'subscription' as const, enabled: true, refreshIntervalMinutes: 15, timeoutSeconds: 12, redistributionAuthorized: true }], candidates: [{ id: 'a', sourceId: 'test', sourceName: 'test', attribution: 'test', uri: '', protocol: 'vless' }], stale: false, acknowledgedWarning: true, automaticFailover: false };
  vi.mocked(api.getCommunitySnapshot).mockResolvedValue({ ...snapshot, stale });
  vi.mocked(api.refreshCommunity).mockResolvedValue(snapshot);
  vi.mocked(api.scanCommunity).mockResolvedValue([{ candidateId: 'a', sourceId: 'test', sourceName: 'test', attribution: 'test', protocol: 'vless', working: true, medianLatencyMs: 120, jitterMs: 2, failureRate: 0, udpAvailable: true, label: 'Fast', score: 100 }]);
  vi.mocked(api.connectCommunity).mockResolvedValue({ ...status, status: 'connected' });
  render(<App />);
  await waitFor(() => expect(api.loadProfile).toHaveBeenCalled());
  fireEvent.click(screen.getByRole('button', { name: 'اتصال سریع رایگان' }));
  const button = screen.getByRole('button', { name: 'اتصال Discord' });
  await waitFor(() => expect(button).toBeEnabled());
  fireEvent.click(button);
  await waitFor(() => expect(api.connectCommunity).toHaveBeenCalledWith(['a']));
  if (stale) {
    expect(vi.mocked(api.refreshCommunity).mock.invocationCallOrder[0]).toBeLessThan(vi.mocked(api.scanCommunity).mock.invocationCallOrder[0]);
  } else {
    expect(api.refreshCommunity).not.toHaveBeenCalled();
  }
  expect(api.connect).not.toHaveBeenCalled();
});
it('restores a masked profile and saves it before connection', async () => {
  render(<App />);
  await waitFor(() => expect(screen.getByDisplayValue(profile.configLink)).toHaveAttribute('type', 'password'));
  fireEvent.click(screen.getByRole('button', { name: 'اتصال Discord' }));
  await waitFor(() => expect(api.connect).toHaveBeenCalledWith(profile));
  expect(api.saveProfile).toHaveBeenCalledWith(profile);
});
it('honors opt-out and removes the previous stored profile', async () => {
  render(<App />);
  await screen.findByDisplayValue(profile.configLink);
  fireEvent.click(screen.getByRole('checkbox', { name: /کانفیگ برای دفعات بعد/ }));
  fireEvent.click(screen.getByRole('button', { name: 'ذخیره تنظیمات' }));
  await waitFor(() => expect(api.forgetProfile).toHaveBeenCalled());
  expect(api.saveProfile).not.toHaveBeenCalled();
});
it('requires confirmation before forgetting and retains input when cancelled', async () => {
  render(<App />);
  await screen.findByDisplayValue(profile.configLink);
  fireEvent.click(screen.getByRole('button', { name: 'حذف کانفیگ ذخیره‌شده' }));
  expect(api.forgetProfile).not.toHaveBeenCalled();
  fireEvent.click(screen.getByRole('button', { name: 'انصراف' }));
  expect(screen.getByDisplayValue(profile.configLink)).toBeInTheDocument();
  fireEvent.click(screen.getByRole('button', { name: 'حذف کانفیگ ذخیره‌شده' }));
  const confirmDelete = screen.getByRole('button', { name: 'بله، حذف شود' });
  expect(confirmDelete).toHaveFocus();
  fireEvent.click(confirmDelete);
  await waitFor(() => expect(screen.queryByDisplayValue(profile.configLink)).not.toBeInTheDocument());
});
it('does not connect or claim saved when protected storage fails', async () => {
  vi.mocked(api.saveProfile).mockRejectedValue(new Error('storage unavailable'));
  render(<App />);
  await screen.findByDisplayValue(profile.configLink);
  fireEvent.click(screen.getByRole('button', { name: 'اتصال Discord' }));
  await screen.findByText('Error: storage unavailable');
  expect(api.connect).not.toHaveBeenCalled();
});
it('keeps profile data when switching guide and connection views', async () => {
  render(<App />);
  await screen.findByDisplayValue(profile.configLink);
  fireEvent.click(screen.getByRole('button', { name: 'راهنمای شروع' }));
  expect(screen.getByRole('heading', { name: 'راه‌اندازی DisRoute' })).toBeInTheDocument();
  fireEvent.click(screen.getByRole('button', { name: 'بازگشت به اتصال' }));
  expect(screen.getByDisplayValue(profile.configLink)).toBeInTheDocument();
});

it('offers a Discord restart without disconnecting the tunnel', async () => {
  render(<App />);
  await screen.findByDisplayValue(profile.configLink);
  fireEvent.click(screen.getByRole('button', { name: 'اتصال Discord' }));
  const restart = await screen.findByRole('button', { name: 'Restart Discord' });
  fireEvent.click(restart);
  await waitFor(() => expect(api.restartDiscord).toHaveBeenCalledOnce());
  expect(api.disconnect).not.toHaveBeenCalled();
  expect(await screen.findByText('Discord دوباره اجرا شد.')).toBeInTheDocument();
});

it('detects supported proxy protocols and rejects unknown schemes inline', async () => {
  vi.mocked(api.loadProfile).mockResolvedValue(null);
  render(<App />);
  const input = await screen.findByPlaceholderText('vless:// · vmess:// · trojan:// · ss://');
  fireEvent.change(input, { target: { value: 'trojan://secret@example.com:443' } });
  expect(screen.getByText('Trojan')).toBeInTheDocument();
  expect(screen.getByRole('button', { name: 'اتصال Discord' })).toBeEnabled();
  fireEvent.change(input, { target: { value: 'https://example.com/config' } });
  fireEvent.blur(input);
  expect(input).toHaveAttribute('aria-invalid', 'true');
  expect(screen.getByText('این نوع لینک پشتیبانی نمی‌شود.')).toBeInTheDocument();
  expect(screen.getByRole('button', { name: 'اتصال Discord' })).toBeDisabled();
});
