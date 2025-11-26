import { expect, test } from '@playwright/test';

const baseUrl = process.env.FRONTEND_BASE_URL ?? 'http://127.0.0.1:8090';

async function goto(page, path: string) {
  await page.goto(`${baseUrl}${path}`);
}

test('mapping workbench renders validation + history shells', async ({ page }) => {
  await goto(page, '/map');
  await expect(page.locator('form hx-post="/map/paste"')).toBeVisible();
  await expect(page.locator('#validation-summary')).toBeVisible();
  await expect(page.locator('#mapping-history')).toBeVisible();
});

test('observability dashboard shows log panel', async ({ page }) => {
  await goto(page, '/observability');
  await expect(page.locator('#log-panel')).toBeVisible();
  await expect(page.locator('text=Vector usage')).toBeVisible();
});

test('evaluation control center exposes catalog + queue panel', async ({ page }) => {
  await goto(page, '/eval');
  await expect(page.locator('text=Dataset catalog')).toBeVisible();
  await expect(page.locator('#eval-jobs-panel')).toBeVisible();
  await expect(page.locator('#eval-compare-panel')).toBeVisible();
});
