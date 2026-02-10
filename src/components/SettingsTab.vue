<script setup lang="ts">
import { Save } from "lucide-vue-next";

interface HostConfig {
  id: string;
  name: string;
  address: string;
  command: string;
  // ... other fields if needed for settings context
}

interface HostPreset {
  id: string;
  name: string;
  address: string;
  command: string;
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

defineProps<{
  settings: AppSettings;
}>();

defineEmits<{
  (e: "save-settings"): void;
}>();
</script>

<template>
  <div class="max-w-xl space-y-8">
    <section class="space-y-4">
      <h3
        class="text-xs font-bold text-slate-500 uppercase tracking-widest px-1"
      >
        状态栏策略
      </h3>
      <div
        class="bg-[#2d2d2d]/30 border border-white/5 rounded-xl p-4 grid grid-cols-4 gap-2"
      >
        <button
          v-for="s in ['fastest', 'mean', 'worst', 'first']"
          :key="s"
          @click="settings.display_strategy = s as any"
          :class="[
            'py-2 rounded-md text-[10px] font-bold transition-all uppercase',
            settings.display_strategy === s
              ? 'bg-blue-600 text-white'
              : 'bg-white/5 text-slate-500 hover:bg-white/10',
          ]"
        >
          {{
            s === "fastest"
              ? "最优"
              : s === "mean"
              ? "平均值"
              : s === "worst"
              ? "最差"
              : "首选"
          }}
        </button>
      </div>
    </section>

    <section class="space-y-4">
      <h3
        class="text-xs font-bold text-slate-500 uppercase tracking-widest px-1"
      >
        监控频率
      </h3>
      <div
        class="bg-[#2d2d2d]/30 border border-white/5 rounded-xl p-4 grid grid-cols-5 gap-2"
      >
        <button
          v-for="i in [5, 10, 15, 30, 60]"
          :key="i"
          @click="settings.ping_interval = i"
          :class="[
            'py-2 rounded-md font-mono text-xs font-bold transition-all',
            settings.ping_interval === i
              ? 'bg-green-600 text-white'
              : 'bg-white/5 text-slate-500 hover:bg-white/10',
          ]"
        >
          {{ i }}s
        </button>
      </div>
    </section>

    <section class="space-y-4">
      <div class="flex items-center justify-between px-1">
        <h3 class="text-xs font-bold text-slate-500 uppercase tracking-widest">
          通知设置
        </h3>
        <button
          @click="
            settings.enable_notifications = !settings.enable_notifications
          "
          :class="[
            'w-10 h-5 rounded-full p-1 transition-colors flex items-center',
            settings.enable_notifications ? 'bg-blue-600' : 'bg-slate-700',
          ]"
        >
          <div
            :class="[
              'w-3 h-3 rounded-full bg-white shadow-sm transition-transform',
              settings.enable_notifications ? 'translate-x-5' : 'translate-x-0',
            ]"
          ></div>
        </button>
      </div>

      <div
        v-if="settings.enable_notifications"
        class="bg-[#2d2d2d]/30 border border-white/5 rounded-xl p-4 space-y-4"
      >
        <div class="flex gap-4">
          <label class="flex items-center gap-2 cursor-pointer">
            <input
              type="radio"
              value="system"
              v-model="settings.notification_type"
              class="accent-blue-500"
            />
            <span class="text-xs font-bold text-slate-300">系统通知</span>
          </label>
          <label class="flex items-center gap-2 cursor-pointer">
            <input
              type="radio"
              value="bark"
              v-model="settings.notification_type"
              class="accent-blue-500"
            />
            <span class="text-xs font-bold text-slate-300">Bark 推送</span>
          </label>
        </div>

        <div v-if="settings.notification_type === 'bark'" class="space-y-1.5">
          <label class="text-[10px] font-bold text-slate-500 uppercase"
            >Bark URL</label
          >
          <input
            v-model="settings.bark_url"
            placeholder="https://api.day.app/..."
            class="w-full bg-[#1e1e1e] border border-white/10 rounded-lg px-4 py-2 text-xs text-white focus:border-blue-500 outline-none font-mono"
          />
        </div>
      </div>
    </section>

    <section class="space-y-4">
      <h3
        class="text-xs font-bold text-slate-500 uppercase tracking-widest px-1"
      >
        日志级别
      </h3>
      <div
        class="bg-[#2d2d2d]/30 border border-white/5 rounded-xl p-4 grid grid-cols-4 gap-2"
      >
        <button
          v-for="l in ['debug', 'info', 'warn', 'error']"
          :key="l"
          @click="settings.log_level = l as any"
          :class="[
            'py-2 rounded-md text-[10px] font-bold transition-all uppercase',
            settings.log_level === l
              ? 'bg-purple-600 text-white'
              : 'bg-white/5 text-slate-500 hover:bg-white/10',
          ]"
        >
          {{ l }}
        </button>
      </div>
    </section>

    <div class="flex justify-end gap-3 pt-6">
      <button
        @click="$emit('save-settings')"
        class="px-8 py-2.5 bg-blue-600 text-white rounded-md text-xs font-bold shadow-xl hover:bg-blue-500 transition-all flex items-center gap-2"
      >
        <Save class="w-4 h-4" /> 保存并应用设置
      </button>
    </div>
  </div>
</template>
