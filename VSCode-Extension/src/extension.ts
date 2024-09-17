/*
 * Copyright (C) 2019 Intel Corporation.  All rights reserved.
 * SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception
 */

import * as fileSystem from 'fs';
import * as os from 'os';
import * as path from 'path';
import * as vscode from 'vscode';

import { WasmTaskProvider } from './taskProvider';
import { TargetConfigPanel } from './view/TargetConfigPanel';
import {
    writeIntoFile,
    readFromFile,
} from './utilities/directoryUtilities';
import { WasmDebugConfigurationProvider } from './debugConfigurationProvider';
import {
    isLLDBInstalled,
    promptInstallLLDB,
    getWAMRExtensionVersion,
} from './utilities/lldbUtilities';

import { SelectionOfPrompt } from './constants';

let wasmTaskProvider: WasmTaskProvider;
let wasmDebugConfigProvider: WasmDebugConfigurationProvider;
let currentPrjDir = '';
let hasProjOpened = false;

// eslint-disable-next-line @typescript-eslint/explicit-module-boundary-types
export async function activate(context: vscode.ExtensionContext) {
    const extensionPath = context.extensionPath;
    const osPlatform = os.platform();
    const wamrVersion = getWAMRExtensionVersion(context.extensionPath);
    const typeMap = new Map<string, string>();
    const scriptMap = new Map<string, string>();
    /* set relative path of build.bat|sh script */
    const scriptPrefix = 'resource/scripts/';

    let runScript = '',
        debugScript = '',
        runScriptFullPath = '',
        debugScriptFullPath = ''

    /**
     * Provide Build & Run Task with Task Provider instead of "tasks.json"
     */

    if (osPlatform === 'win32') {
        runScript = scriptPrefix.concat('run.bat');
        debugScript = scriptPrefix.concat('boot_debugger_server.bat');
    } else if (osPlatform === 'linux' || osPlatform === 'darwin') {
        runScript = scriptPrefix.concat('run.sh');
        debugScript = scriptPrefix.concat('boot_debugger_server.sh');
    }

    runScriptFullPath = path.join(extensionPath, runScript);
    debugScriptFullPath = path.join(extensionPath, debugScript);

    scriptMap.set('runScript', runScriptFullPath);
    scriptMap.set('debugScript', debugScriptFullPath);

    typeMap.set('Run', 'Run');
    typeMap.set('Debug', 'Debug');

    wasmTaskProvider = new WasmTaskProvider(typeMap, scriptMap, wamrVersion);

    vscode.tasks.registerTaskProvider('wasm', wasmTaskProvider);

    if (vscode.workspace.workspaceFolders?.[0]) {
        if (osPlatform === 'win32') {
            currentPrjDir = vscode.workspace.workspaceFolders?.[0].uri
                .fsPath as string;
        } else if (osPlatform === 'linux' || osPlatform === 'darwin') {
            currentPrjDir = vscode.workspace.workspaceFolders?.[0].uri
                .path as string;
        }
    }
    hasProjOpened = currentPrjDir.length > 0;
    vscode.commands.executeCommand('setContext', 'ext.hasProjOpened', hasProjOpened)

    /* register debug configuration */
    wasmDebugConfigProvider = new WasmDebugConfigurationProvider(
        context.extensionPath
    );

    vscode.debug.registerDebugConfigurationProvider(
        'wamr-debug',
        wasmDebugConfigProvider
    );

    if (readFromConfigFile() !== '') {
        const configData = JSON.parse(readFromConfigFile());

        if (Object.keys(configData['buildArgs']).length !== 0) {
            TargetConfigPanel.buildArgs = configData['buildArgs'];
        }
    }

    const disposableTargetConfig = vscode.commands.registerCommand(
        'wamride.targetConfig',
        () => {
            TargetConfigPanel.render(context);
        }
    );

    const disposableDebug = vscode.commands.registerCommand(
        'wamride.debug',
        async () => {
            /* we should check again whether the user installed lldb, as this can be skipped during activation */
            try {
                if (!isLLDBInstalled(context.extensionPath)) {
                    /**NOTE - if users select to skip install,
                     *        we should return rather than continue
                     *        the execution
                     */
                    if (
                        (await promptInstallLLDB(context.extensionPath)) ===
                        SelectionOfPrompt.skip
                    ) {
                        return;
                    }
                }

            } catch (e) {
                vscode.window.showWarningMessage((e as Error).message);
                return;
            }

            /* refuse to debug if build process failed */
            if (!checkIfBuildSuccess()) {
                vscode.window.showErrorMessage('Debug failed', {
                    modal: true,
                    detail: 'Can not find WASM binary, please build WASM firstly.',
                });
                return;
            }

            /* show debug view */
            vscode.commands.executeCommand('workbench.view.debug');

            /* should destroy the wasm-debug-server-ctr before debugging */
            vscode.commands.executeCommand(
                    'workbench.action.tasks.runTask',
                    'Debug: Wasm'
                )
                .then(() => {
                    vscode.debug
                        .startDebugging(
                            undefined,
                            wasmDebugConfigProvider.getDebugConfig()
                        )
                        .then(() => {
                            /* register to listen debug session finish event */
                            const disposableAft =
                                vscode.debug.onDidTerminateDebugSession(
                                    s => {
                                        if (
                                            s.type !==
                                            'wamr-debug'
                                        ) {
                                            return;
                                        }

                                        /* execute the task to kill the terminal */
                                        vscode.commands.executeCommand(
                                            'workbench.action.terminal.kill',
                                            'Debug: Wasm'
                                        );

                                        disposableAft.dispose();
                                    }
                                );
                        });
                });
        }
    );

    const disposableRun = vscode.commands.registerCommand(
        'wamride.run',
        async () => {
            /* refuse to debug if build process failed */
            if (!checkIfBuildSuccess()) {
                vscode.window.showErrorMessage('Debug failed', {
                    modal: true,
                    detail: 'Can not find WASM binary, please build WASM firstly.',
                });
                return;
            }

            vscode.commands.executeCommand(
                    'workbench.action.tasks.runTask',
                    'Run: Wasm');
        }
    );

    context.subscriptions.push(
        disposableTargetConfig,
        disposableRun,
        disposableDebug
    );

    try {
        if (!isLLDBInstalled(context.extensionPath)) {
            await promptInstallLLDB(context.extensionPath);
        }
    } catch (e) {
        vscode.window.showWarningMessage((e as Error).message);
    }
}

interface BuildArgs {
    outputFileName: string;
    initMemorySize: string;
    maxMemorySize: string;
    stackSize: string;
    exportedSymbols: string;
    hostManagedHeapSize: string;
}

/**
 * @param: includePathArr
 * @param: excludeFileArr
 *   Get current includePathArr and excludeFileArr from the json string that
 *   will be written into compilation_config.json
 */
export function writeIntoConfigFile(
    buildArgs?: BuildArgs
): void {
    const jsonStr = JSON.stringify(
        {
            buildArgs: buildArgs ? buildArgs : '{}',
        },
        null,
        '\t'
    );

    const prjConfigDir = path.join(currentPrjDir, '.wamr');
    const configFilePath = path.join(prjConfigDir, 'compilation_config.json');
    writeIntoFile(configFilePath, jsonStr);
}

export function readFromConfigFile(): string {
    const prjConfigDir = path.join(currentPrjDir, '.wamr');
    const configFilePath = path.join(prjConfigDir, 'compilation_config.json');
    return readFromFile(configFilePath);
}

function checkIfBuildSuccess(): boolean {
    try {
        let wasmExist = false;
        const entries = fileSystem.readdirSync(
            path.join(currentPrjDir, 'build'),
            {
                withFileTypes: true,
            }
        );

        entries.map(e => {
            if (e.name.match('(.wasm)$')) {
                wasmExist = true;
            }
        });

        return wasmExist;
    } catch {
        return false;
    }
}
