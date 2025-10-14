#!/usr/bin/env node

// 测试 Libre Browser 图标生成功能的脚本

// 模拟 Tauri 命令调用
async function testIconGeneration() {
    console.log('开始测试图标生成功能...');

    // 创建一个测试浏览器配置
    const testBrowser = {
        id: 'TEST-' + Date.now().toString().slice(-6),
        name: 'Test Browser',
        engineVersion: '141'
    };

    console.log(`创建测试浏览器: ${testBrowser.name} (${testBrowser.id})`);

    // 这里我们需要通过前端界面来触发 browser_open 命令
    // 由于我们无法直接调用 Tauri 命令，我们需要通过 HTTP API 来测试
    console.log('请手动在 Libre Browser 界面中：');
    console.log(`1. 点击"新建浏览器"`);
    console.log(`2. 输入名称: ${testBrowser.name}`);
    console.log(`3. 选择引擎版本: ${testBrowser.engineVersion}`);
    console.log(`4. 点击创建`);
    console.log(`5. 在浏览器列表中找到新创建的浏览器并点击"启动"`);

    console.log('\n启动后，检查以下位置是否生成了自定义图标：');
    console.log('~/Library/Application Support/libre-browser/custom_icons/');
    console.log('~/Library/Application Support/libre-browser/custom_apps/');

    console.log('\n如果成功，您应该能在 macOS Dock 中看到带有自定义颜色的浏览器图标。');
}

testIconGeneration().catch(console.error);