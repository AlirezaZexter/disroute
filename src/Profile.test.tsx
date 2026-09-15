import "@testing-library/jest-dom/vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import * as api from "./api";
import App from "./App";

vi.mock('./api', () => ({ getStatus: vi.fn(), loadProfile: vi.fn(), saveProfile: vi.fn(), forgetProfile: vi.fn(), connect: vi.fn(), disconnect: vi.fn(), hideToTray: vi.fn().mockResolvedValue(undefined), restartDiscord: vi.fn() }));
const profile = { name: 'My route', vlessLink: 'vless://00000000-0000-4000-8000-000000000000@example.com:443?security=tls' };
const status = { status: 'disconnected' as const, engineReady: true, isElevated: true, message: 'ready' };
beforeEach(() => {
  vi.resetAllMocks();
  vi.mocked(api.getStatus).mockResolvedValue(status);
  vi.mocked(api.loadProfile).mockResolvedValue(profile);
  vi.mocked(api.saveProfile).mockResolvedValue();
  vi.mocked(api.forgetProfile).mockResolvedValue();
  vi.mocked(api.connect).mockResolvedValue({ ...status, status: 'connected' });
  vi.mocked(api.restartDiscord).mockResolvedValue('Discord دوباره اجرا شد.');
});
afterEach(cleanup);
it('restores a masked profile and saves it before connection', async () => {
  render(<App />);
  await waitFor(() => expect(screen.getByDisplayValue(profile.vlessLink)).toHaveAttribute('type', 'password'));
  fireEvent.click(screen.getByRole('button', { name: 'اتصال Discord' }));
  await waitFor(() => expect(api.connect).toHaveBeenCalledWith(profile));
  expect(api.saveProfile).toHaveBeenCalledWith(profile);
});
it('honors opt-out and removes the previous stored profile', async () => {
  render(<App />);
  await screen.findByDisplayValue(profile.vlessLink);
  fireEvent.click(screen.getByRole('checkbox', { name: /کانفیگ برای دفعات بعد/ }));
  fireEvent.click(screen.getByRole('button', { name: 'ذخیره تنظیمات' }));
  await waitFor(() => expect(api.forgetProfile).toHaveBeenCalled());
  expect(api.saveProfile).not.toHaveBeenCalled();
});
it('requires confirmation before forgetting and retains input when cancelled', async () => {
  render(<App />);
  await screen.findByDisplayValue(profile.vlessLink);
  fireEvent.click(screen.getByRole('button', { name: 'حذف کانفیگ ذخیره‌شده' }));
  expect(api.forgetProfile).not.toHaveBeenCalled();
  fireEvent.click(screen.getByRole('button', { name: 'انصراف' }));
  expect(screen.getByDisplayValue(profile.vlessLink)).toBeInTheDocument();
  fireEvent.click(screen.getByRole('button', { name: 'حذف کانفیگ ذخیره‌شده' }));
  fireEvent.click(screen.getByRole('button', { name: 'بله، حذف شود' }));
  await waitFor(() => expect(screen.queryByDisplayValue(profile.vlessLink)).not.toBeInTheDocument());
});
it('does not connect or claim saved when protected storage fails', async () => {
  vi.mocked(api.saveProfile).mockRejectedValue(new Error('storage unavailable'));
  render(<App />);
  await screen.findByDisplayValue(profile.vlessLink);
  fireEvent.click(screen.getByRole('button', { name: 'اتصال Discord' }));
  await screen.findByText('Error: storage unavailable');
  expect(api.connect).not.toHaveBeenCalled();
});
it('keeps profile data when switching guide and connection views', async () => {
  render(<App />);
  await screen.findByDisplayValue(profile.vlessLink);
  fireEvent.click(screen.getByRole('button', { name: 'راهنمای شروع' }));
  expect(screen.getByRole('heading', { name: 'راه‌اندازی DisRoute' })).toBeInTheDocument();
  fireEvent.click(screen.getByRole('button', { name: 'اتصال و پروفایل' }));
  expect(screen.getByDisplayValue(profile.vlessLink)).toBeInTheDocument();
});

it('offers a Discord restart without disconnecting the tunnel', async () => {
  render(<App />);
  await screen.findByDisplayValue(profile.vlessLink);
  fireEvent.click(screen.getByRole('button', { name: 'اتصال Discord' }));
  const restart = await screen.findByRole('button', { name: 'Restart Discord' });
  fireEvent.click(restart);
  await waitFor(() => expect(api.restartDiscord).toHaveBeenCalledOnce());
  expect(api.disconnect).not.toHaveBeenCalled();
  expect(await screen.findByText('Discord دوباره اجرا شد.')).toBeInTheDocument();
});
