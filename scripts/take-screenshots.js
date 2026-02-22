const { chromium } = require('playwright');
const path = require('path');
const fs = require('fs');

const SCREENSHOTS_DIR = path.join(__dirname, '..', 'docs', 'screenshots');
const BASE_URL = 'http://localhost:8080';

// Ensure screenshots directory exists
if (!fs.existsSync(SCREENSHOTS_DIR)) {
  fs.mkdirSync(SCREENSHOTS_DIR, { recursive: true });
}

async function takeScreenshots() {
  const browser = await chromium.launch({ headless: true });

  // Desktop screenshots
  const desktopContext = await browser.newContext({
    viewport: { width: 1280, height: 800 },
    deviceScaleFactor: 2,
  });
  const desktopPage = await desktopContext.newPage();

  // Login first
  console.log('Logging in...');
  await desktopPage.goto(`${BASE_URL}/login`);
  await desktopPage.waitForTimeout(1000);

  // Fill login form
  await desktopPage.fill('input[type="text"]', 'admin');
  await desktopPage.fill('input[type="password"]', 'admin');
  await desktopPage.click('button[type="submit"]');
  await desktopPage.waitForTimeout(2000);

  // Dashboard
  console.log('Taking dashboard screenshot...');
  await desktopPage.goto(`${BASE_URL}/dashboard`);
  await desktopPage.waitForTimeout(2000);
  await desktopPage.screenshot({
    path: path.join(SCREENSHOTS_DIR, 'dashboard.png'),
    fullPage: false
  });

  // Services
  console.log('Taking services screenshot...');
  await desktopPage.goto(`${BASE_URL}/services`);
  await desktopPage.waitForTimeout(2000);
  await desktopPage.screenshot({
    path: path.join(SCREENSHOTS_DIR, 'services.png'),
    fullPage: false
  });

  // Flakes
  console.log('Taking flakes screenshot...');
  await desktopPage.goto(`${BASE_URL}/flakes`);
  await desktopPage.waitForTimeout(2000);
  await desktopPage.screenshot({
    path: path.join(SCREENSHOTS_DIR, 'flakes.png'),
    fullPage: false
  });

  // Rebuild
  console.log('Taking rebuild screenshot...');
  await desktopPage.goto(`${BASE_URL}/rebuild`);
  await desktopPage.waitForTimeout(2000);
  await desktopPage.screenshot({
    path: path.join(SCREENSHOTS_DIR, 'rebuild.png'),
    fullPage: false
  });

  // Nix Operations
  console.log('Taking nix-operations screenshot...');
  await desktopPage.goto(`${BASE_URL}/nix`);
  await desktopPage.waitForTimeout(2000);
  await desktopPage.screenshot({
    path: path.join(SCREENSHOTS_DIR, 'nix-operations.png'),
    fullPage: false
  });

  // Secrets
  console.log('Taking secrets screenshot...');
  await desktopPage.goto(`${BASE_URL}/secrets`);
  await desktopPage.waitForTimeout(2000);
  await desktopPage.screenshot({
    path: path.join(SCREENSHOTS_DIR, 'secrets.png'),
    fullPage: false
  });

  await desktopContext.close();

  // Mobile screenshot
  console.log('Taking mobile screenshot...');
  const mobileContext = await browser.newContext({
    viewport: { width: 390, height: 844 },
    deviceScaleFactor: 2,
    isMobile: true,
  });
  const mobilePage = await mobileContext.newPage();

  // Login on mobile
  await mobilePage.goto(`${BASE_URL}/login`);
  await mobilePage.waitForTimeout(1000);
  await mobilePage.fill('input[type="text"]', 'admin');
  await mobilePage.fill('input[type="password"]', 'admin');
  await mobilePage.click('button[type="submit"]');
  await mobilePage.waitForTimeout(2000);

  await mobilePage.goto(`${BASE_URL}/dashboard`);
  await mobilePage.waitForTimeout(2000);
  await mobilePage.screenshot({
    path: path.join(SCREENSHOTS_DIR, 'mobile.png'),
    fullPage: false
  });

  await mobileContext.close();
  await browser.close();

  console.log('Screenshots saved to docs/screenshots/');
}

takeScreenshots().catch(console.error);
