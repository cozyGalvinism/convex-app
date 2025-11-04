import { invoke } from "@tauri-apps/api/core";

export type Instance = {
    id: string;
    name: string;
    icon_b64?: string;
    last_launch_ms?: number;
    total_time_ms?: number;
};

export type ScanResult = {
    instances: Instance[];
    groups: Record<string, string[]>;
};

export async function scan(root?: string): Promise<ScanResult> {
    return invoke("scan_prism", { rootOverride: root });
}

export async function launchInstance(instanceId: string, root?: string): Promise<void> {
    return invoke("launch_instance", { instanceId, rootOverride: root });
}

export async function showInstance(instanceId: string, root?: string): Promise<void> {
    return invoke("show_instance", { instanceId, rootOverride: root });
}
