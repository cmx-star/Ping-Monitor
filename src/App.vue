<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

// Components
import AppHeader from "./components/Header.vue";
import MonitorTab from "./components/MonitorTab.vue";
import StatisticsTab from "./components/StatisticsTab.vue";
import HostManagementTab from "./components/HostManagementTab.vue";
import LogsTab from "./components/LogsTab.vue";
import SettingsTab from "./components/SettingsTab.vue";
import AddHostModal from "./components/AddHostModal.vue";

// Types
interface DisplayRule {
  id: string;
  condition: "less" | "greater";
  threshold: number;
  label: string;
  enabled: boolean;
}

interface HostConfig {
  id: string;
  name: string;
  address: string;
  command: string;
  display_rules: DisplayRule[];
}

interface HostPreset {
  id: string;
  name: string;
  address: string;
  command: string;
}

interface PingStats {
  host_id: string;
  current: number;
  mean: number;
  std_dev: number;
  median: number;
  min: number;
  max: number;
  total_pings: number;
  successful_pings: number;
  failed_pings: number;
  packet_loss_rate: number;
  success_rate: number;
  bytes_sent: number;
  bytes_received: number;
  peaks_count: number;
  peaks_per_minute: number;
  peaks_mean: number;
  peaks_max: number;
  last_peak: string | null;
  status: string;
  labels: string[];
  start_time: string;
}

interface AppSettings {
  hosts: HostConfig[];
  ping_interval: number;
  auto_start: boolean;
  notification_type: string;
  bark_url: string;
  display_strategy: "mean" | "worst" | "fastest" | "first";
  show_latency: boolean;
  show_labels: boolean;
  log_level: "debug" | "info" | "warn" | "error";
  enable_notifications: boolean;
  presets: HostPreset[];
}

interface LogEntry {
  id: string;
  timestamp: Date;
  level: "DEBUG" | "INFO" | "WARN" | "ERROR";
  message: string;
  host?: string;
}

// --- State ---

const activeTab = ref(0); // 0: 监控, 1: 统计, 2: 主机管理, 3: 日志, 4: 设置
const isRunning = ref(true);
const hosts = ref<HostConfig[]>([]);
const hostStats = ref<Record<string, PingStats>>({});
const hostHistory = ref<Record<string, number[]>>({});
const selectedHostId = ref<string | null>(null);

// Modal state
const showAddHostModal = ref(false);
const showEditHostModal = ref(false);
const editingHost = ref<Partial<HostConfig>>({});
const newHost = ref<Partial<HostConfig>>({
  name: "",
  address: "",
  command: "",
  display_rules: [
    {
      id: crypto.randomUUID(),
      condition: "less",
      threshold: 50,
      label: "P2P",
      enabled: true,
    },
    {
      id: crypto.randomUUID(),
      condition: "greater",
      threshold: 50,
      label: "转发",
      enabled: true,
    },
  ],
});

// Settings state
const settings = ref<AppSettings>({
  hosts: [],
  ping_interval: 5,
  auto_start: false,
  notification_type: "system",
  bark_url: "",
  display_strategy: "first",
  show_latency: true,
  show_labels: true,
  log_level: "info",
  enable_notifications: true,
  presets: [],
});

// Toast state
const showToast = ref(false);
const toastMessage = ref("");
const toastType = ref<"success" | "error">("success");
let toastTimeout: number | undefined;

const triggerToast = (
  message: string,
  type: "success" | "error" = "success"
) => {
  toastMessage.value = message;
  toastType.value = type;
  showToast.value = true;

  if (toastTimeout) clearTimeout(toastTimeout);
  toastTimeout = window.setTimeout(() => {
    showToast.value = false;
  }, 3000);
};

const logs = ref<LogEntry[]>([]); // To be populated from backend events or logs view logic

// --- Methods ---

const loadHosts = async () => {
  hosts.value = await invoke("get_hosts");
  if (hosts.value.length > 0 && !selectedHostId.value) {
    selectedHostId.value = hosts.value[0].id;
  }
};

const loadSettings = async () => {
  settings.value = await invoke("get_settings");
};

const saveSettings = async () => {
  try {
    await invoke("apply_settings", { newSettings: settings.value });
    triggerToast("设置已保存并应用", "success");
  } catch (e) {
    console.error("Failed to save settings:", e);
    triggerToast(`保存失败: ${e}`, "error");
  }
};

const startMonitoring = async (hostId: string) => {
  try {
    console.log(`[Frontend] Starting monitoring for ${hostId}`);

    const host = hosts.value.find((h) => h.id === hostId);
    logs.value.unshift({
      id: crypto.randomUUID(),
      timestamp: new Date(),
      level: "INFO",
      message: `Starting monitoring...`,
      host: host?.name || hostId,
    });

    await invoke("start_monitoring", { hostId });
  } catch (e) {
    console.error(`Failed to start ${hostId}:`, e);
    const host = hosts.value.find((h) => h.id === hostId);
    logs.value.unshift({
      id: crypto.randomUUID(),
      timestamp: new Date(),
      level: "ERROR",
      message: `Failed to start monitoring: ${e}`,
      host: host?.name || hostId,
    });
  }
};

const stopMonitoring = async (hostId: string) => {
  const host = hosts.value.find((h) => h.id === hostId);
  logs.value.unshift({
    id: crypto.randomUUID(),
    timestamp: new Date(),
    level: "INFO",
    message: `Stopping monitoring...`,
    host: host?.name || hostId,
  });
  await invoke("stop_monitoring", { hostId });
};

const startAll = async () => {
  await invoke("start_all");
  isRunning.value = true;
};

const stopAll = async () => {
  await invoke("stop_all");
  isRunning.value = false;
};

const openAddModal = () => {
  newHost.value = {
    name: "",
    address: "",
    command: "",
    display_rules: [
      {
        id: crypto.randomUUID(),
        condition: "less",
        threshold: 50,
        label: "P2P",
        enabled: true,
      },
      {
        id: crypto.randomUUID(),
        condition: "greater",
        threshold: 50,
        label: "转发",
        enabled: true,
      },
    ],
  };
  showAddHostModal.value = true;
};

const openEditModal = (host: HostConfig) => {
  editingHost.value = JSON.parse(JSON.stringify(host));
  showEditHostModal.value = true;
};

const confirmUpdateHost = async () => {
  if (!editingHost.value.id) return;
  await invoke("update_host", { config: editingHost.value });
  showEditHostModal.value = false;
  await loadHosts();
};

const confirmAddHost = async () => {
  if (!newHost.value.name || !newHost.value.address) return;
  const config = {
    ...newHost.value,
    id: crypto.randomUUID(),
  } as HostConfig;

  await invoke("add_host", { config });
  await loadHosts();
  showAddHostModal.value = false;
  await startMonitoring(config.id);
};

const deleteHost = async (hostId: string) => {
  await stopMonitoring(hostId);
  await invoke("remove_host", { hostId });
  await loadHosts();
};

const handleSelectHost = (id: string) => {
  selectedHostId.value = id;
  activeTab.value = 1; // Switch to stats tab
};

const exportLogs = () => {
  if (logs.value.length === 0) return;

  const headers = ["Timestamp", "Level", "Host", "Message"];
  const csvContent = [
    headers.join(","),
    ...logs.value.map((log) => {
      const timestamp = new Date(log.timestamp).toISOString();
      const host = log.host || "-";
      // Escape quotes in message
      const message = `"${log.message.replace(/"/g, '""')}"`;
      return `${timestamp},${log.level},${host},${message}`;
    }),
  ].join("\n");

  const blob = new Blob([csvContent], { type: "text/csv;charset=utf-8;" });
  const link = document.createElement("a");
  const url = URL.createObjectURL(blob);
  link.setAttribute("href", url);
  link.setAttribute(
    "download",
    `ping_monitor_logs_${new Date()
      .toISOString()
      .slice(0, 19)
      .replace(/:/g, "-")}.csv`
  );
  link.style.visibility = "hidden";
  document.body.appendChild(link);
  link.click();
  document.body.removeChild(link);
};

// Lifecycle
onMounted(async () => {
  console.log("[Frontend] Initializing global ping-stats listener");
  listen<PingStats>("ping-stats", (event) => {
    const stats = event.payload;
    const hostId = stats.host_id;
    // Force reactivity by creating a new object reference
    hostStats.value = {
      ...hostStats.value,
      [hostId]: stats,
    };

    // Fix reactivity for chart: Create new array reference
    const currentHistory = hostHistory.value[hostId] || [];
    const newHistory = [...currentHistory, stats.current];
    if (newHistory.length > 100) newHistory.shift();

    hostHistory.value = {
      ...hostHistory.value,
      [hostId]: newHistory,
    };

    // Generate Log Entry
    const hostName = hosts.value.find((h) => h.id === hostId)?.name || hostId;
    let logEntry: LogEntry | null = null;

    if (
      stats.status === "Unusable" ||
      (stats.failed_pings > 0 && stats.current === 0)
    ) {
      // Timeout / Error
      logEntry = {
        id: crypto.randomUUID(),
        timestamp: new Date(),
        level: "ERROR",
        message: `Request timed out`,
        host: hostName,
      };
    } else if (stats.current > 100) {
      // Warn
      logEntry = {
        id: crypto.randomUUID(),
        timestamp: new Date(),
        level: "WARN",
        message: `High latency detected: ${stats.current.toFixed(1)}ms`,
        host: hostName,
      };
    } else {
      // success - optionally log meaningful info or just periodic
      // To avoid spamming logs, maybe only log state changes or periodic?
      // For now, let's log successful pings as INFO/DEBUG
      logEntry = {
        id: crypto.randomUUID(),
        timestamp: new Date(),
        level: "INFO",
        message: `Reply from ${stats.host_id}: time=${stats.current.toFixed(
          1
        )}ms`,
        host: hostName,
      };
    }

    if (logEntry) {
      logs.value.unshift(logEntry);
      if (logs.value.length > 1000) logs.value.pop();
    }
  });

  await loadHosts();
  await loadSettings();
  await startAll();
});
</script>

<template>
  <div
    class="min-h-screen bg-[#1e1e1e] text-[#d1d1d1] font-sans selection:bg-blue-500/30 flex flex-col overflow-hidden"
  >
    <!-- Header -->
    <AppHeader
      :is-running="isRunning"
      @toggle="isRunning ? stopAll() : startAll()"
    />

    <!-- Navigation Tabs -->
    <nav
      class="h-12 bg-[#2d2d2d]/30 border-b border-black/20 flex px-4 shrink-0"
    >
      <button
        v-for="(item, index) in ['监控', '统计', '主机管理', '日志', '设置']"
        :key="index"
        @click="activeTab = index"
        :class="[
          'px-10 h-full text-xs font-semibold transition-all border-b-2',
          activeTab === index
            ? 'text-white border-blue-500 bg-white/5'
            : 'text-slate-500 border-transparent hover:text-slate-300',
        ]"
      >
        {{ item }}
      </button>
    </nav>

    <!-- Main Content -->
    <main class="flex-1 overflow-y-auto p-8 bg-[#1e1e1e]">
      <div class="max-w-6xl mx-auto">
        <Transition name="fade-slide" mode="out-in">
          <MonitorTab
            v-if="activeTab === 0"
            :hosts="hosts"
            :host-stats="hostStats"
            @add-host="openAddModal"
            @select-host="handleSelectHost"
          />

          <StatisticsTab
            v-else-if="activeTab === 1"
            :hosts="hosts"
            :host-stats="hostStats"
            :host-history="hostHistory"
            v-model:selectedHostId="selectedHostId"
          />

          <HostManagementTab
            v-else-if="activeTab === 2"
            :hosts="hosts"
            :presets="settings.presets || []"
            @add-host="openAddModal"
            @edit-host="openEditModal"
            @delete-host="deleteHost"
            @add-preset="() => {}"
          />

          <LogsTab
            v-else-if="activeTab === 3"
            :logs="logs"
            @clear-logs="logs = []"
            @export-logs="exportLogs"
          />

          <SettingsTab
            v-else-if="activeTab === 4"
            :settings="settings"
            @save-settings="saveSettings"
          />
        </Transition>
      </div>
    </main>

    <!-- Footer -->
    <footer
      class="h-8 bg-[#1a1a1a] border-t border-white/5 flex items-center justify-center text-[9px] text-slate-600 font-bold uppercase tracking-[0.2em]"
    >
      Ping Monitor v2.0.24 &copy; 2026 Developed by Antigravity
    </footer>

    <!-- Modals -->
    <AddHostModal
      :show="showAddHostModal"
      :is-edit="false"
      v-model="newHost"
      @close="showAddHostModal = false"
      @confirm="confirmAddHost"
    />

    <AddHostModal
      :show="showEditHostModal"
      :is-edit="true"
      v-model="editingHost"
      @close="showEditHostModal = false"
      @confirm="confirmUpdateHost"
    />

    <!-- Toast Notification -->
    <Transition name="toast">
      <div
        v-if="showToast"
        :class="[
          'fixed top-6 left-1/2 -translate-x-1/2 px-6 py-3 rounded-xl shadow-2xl flex items-center gap-3 z-50 border backdrop-blur-md',
          toastType === 'success'
            ? 'bg-green-500/10 border-green-500/20 text-green-400'
            : 'bg-red-500/10 border-red-500/20 text-red-400',
        ]"
      >
        <div
          :class="[
            'w-2 h-2 rounded-full',
            toastType === 'success' ? 'bg-green-500' : 'bg-red-500',
          ]"
        ></div>
        <span class="text-xs font-bold">{{ toastMessage }}</span>
      </div>
    </Transition>
  </div>
</template>

<style>
@import url("https://fonts.googleapis.com/css2?family=Plus+Jakarta+Sans:wght@200;300;400;500;600;700;800&family=JetBrains+Mono:wght@400;700&display=swap");

:root {
  font-family: "Plus Jakarta Sans", sans-serif;
  color-scheme: dark;
}

::-webkit-scrollbar {
  width: 6px;
}
::-webkit-scrollbar-track {
  background: transparent;
}
::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.05);
  border-radius: 10px;
}
::-webkit-scrollbar-thumb:hover {
  background: rgba(255, 255, 255, 0.1);
}

.animate-spin-slow {
  animation: spin 8s linear infinite;
}

@keyframes spin {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}

.font-mono {
  font-family: "JetBrains Mono", monospace;
}

input::placeholder {
  color: #475569;
}

/* Transitions */
.fade-slide-enter-active,
.fade-slide-leave-active {
  transition: all 0.3s cubic-bezier(0.16, 1, 0.3, 1);
}

.fade-slide-enter-from {
  opacity: 0;
  transform: translateX(20px);
}

.fade-slide-leave-to {
  opacity: 0;
  transform: translateX(-20px);
}

.modal-enter-active,
.modal-leave-active {
  transition: all 0.3s cubic-bezier(0.16, 1, 0.3, 1);
}

.modal-enter-from,
.modal-leave-to {
  opacity: 0;
  transform: scale(0.95) translateY(10px);
}

/* Toast Transition */
.toast-enter-active,
.toast-leave-active {
  transition: all 0.3s cubic-bezier(0.16, 1, 0.3, 1);
}

.toast-enter-from,
.toast-leave-to {
  opacity: 0;
  transform: translate(-50%, -20px);
}
</style>
