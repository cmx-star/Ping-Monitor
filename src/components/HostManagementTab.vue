<script setup lang="ts">
import { ref } from "vue";
import { Edit, Trash2 } from "lucide-vue-next";

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

defineProps<{
  hosts: HostConfig[];
  presets: HostPreset[];
}>();

defineEmits<{
  (e: "add-host"): void;
  (e: "edit-host", host: HostConfig): void;
  (e: "delete-host", id: string): void;
  (e: "add-preset"): void; // Placeholder for now
}>();

const activeSection = ref<"saved" | "presets">("saved");
</script>

<template>
  <div class="space-y-6">
    <div class="flex justify-between items-center">
      <div class="flex bg-[#2d2d2d]/50 p-1 rounded-lg border border-white/5">
        <button
          @click="activeSection = 'saved'"
          :class="[
            'px-4 py-1.5 rounded-md text-xs font-bold transition-all',
            activeSection === 'saved'
              ? 'bg-blue-600 text-white shadow-lg'
              : 'text-slate-500 hover:text-slate-300',
          ]"
        >
          已保存主机 ({{ hosts.length }})
        </button>
        <button
          @click="activeSection = 'presets'"
          :class="[
            'px-4 py-1.5 rounded-md text-xs font-bold transition-all',
            activeSection === 'presets'
              ? 'bg-blue-600 text-white shadow-lg'
              : 'text-slate-500 hover:text-slate-300',
          ]"
        >
          预设 ({{ presets.length }})
        </button>
      </div>

      <button
        @click="$emit('add-host')"
        class="px-4 py-1.5 bg-blue-600 hover:bg-blue-500 text-white rounded-md text-xs font-bold transition-all shadow-lg"
      >
        添加新地址
      </button>
    </div>

    <!-- Saved Hosts Table -->
    <div
      v-if="activeSection === 'saved'"
      class="bg-[#2d2d2d]/30 rounded-xl border border-white/5 overflow-hidden"
    >
      <table class="w-full text-left">
        <thead
          class="bg-[#2d2d2d]/50 text-[10px] font-bold text-slate-500 uppercase tracking-widest border-b border-white/5"
        >
          <tr>
            <th class="px-6 py-3">标识名称</th>
            <th class="px-6 py-3">IP / 域名</th>
            <th class="px-6 py-3">阈值标签</th>
            <th class="px-6 py-3 text-right">管理</th>
          </tr>
        </thead>
        <tbody class="divide-y divide-white/5">
          <tr
            v-for="host in hosts"
            :key="host.id"
            class="hover:bg-white/5 transition-colors"
          >
            <td class="px-6 py-4 font-bold text-white text-sm">
              {{ host.name }}
            </td>
            <td class="px-6 py-4 text-slate-400 font-mono text-xs">
              {{ host.address }}
            </td>
            <td class="px-6 py-4">
              <div class="flex gap-2">
                <span
                  v-for="rule in host.display_rules"
                  :key="rule.id"
                  class="px-2 py-0.5 bg-white/5 rounded border border-white/5 text-[9px] font-bold text-slate-500"
                >
                  {{ rule.label }} ({{ rule.condition === "less" ? "<" : ">"
                  }}{{ rule.threshold }}ms)
                </span>
              </div>
            </td>
            <td class="px-6 py-4 text-right">
              <div class="flex justify-end gap-2">
                <button
                  @click="$emit('edit-host', host)"
                  class="p-2 text-slate-400 hover:text-blue-400 hover:bg-blue-500/10 rounded-md transition-all"
                >
                  <Edit class="w-4 h-4" />
                </button>
                <button
                  @click="$emit('delete-host', host.id)"
                  class="p-2 text-slate-400 hover:text-red-400 hover:bg-red-500/10 rounded-md transition-all"
                >
                  <Trash2 class="w-4 h-4" />
                </button>
              </div>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- Presets Table -->
    <div
      v-else
      class="bg-[#2d2d2d]/30 rounded-xl border border-white/5 overflow-hidden"
    >
      <table class="w-full text-left">
        <thead
          class="bg-[#2d2d2d]/50 text-[10px] font-bold text-slate-500 uppercase tracking-widest border-b border-white/5"
        >
          <tr>
            <th class="px-6 py-3">预设名称</th>
            <th class="px-6 py-3">地址</th>
            <th class="px-6 py-3 text-right">操作</th>
          </tr>
        </thead>
        <tbody class="divide-y divide-white/5">
          <tr
            v-for="preset in presets"
            :key="preset.id"
            class="hover:bg-white/5 transition-colors"
          >
            <td class="px-6 py-4 font-bold text-white text-sm">
              {{ preset.name }}
            </td>
            <td class="px-6 py-4 text-slate-400 font-mono text-xs">
              {{ preset.address }}
            </td>
            <td class="px-6 py-4 text-right">
              <!-- Preset management actions can be added here -->
              <span class="text-xs text-slate-600 italic">默认预设不可用</span>
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>
