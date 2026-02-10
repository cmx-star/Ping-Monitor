<script setup lang="ts">
import { Activity, Plus } from "lucide-vue-next";

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

interface PingStats {
  host_id: string;
  current: number;
  mean: number;
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
}

defineProps<{
  hosts: HostConfig[];
  hostStats: Record<string, PingStats>;
}>();

defineEmits<{
  (e: "add-host"): void;
  (e: "select-host", id: string): void;
}>();
</script>

<template>
  <div class="space-y-6">
    <div class="flex justify-between items-center mb-6">
      <h2 class="text-sm font-bold text-slate-400">
        监控中主机 ({{ hosts.length }})
      </h2>
      <button
        @click="$emit('add-host')"
        class="px-3 py-1 bg-blue-600/10 text-blue-400 border border-blue-500/20 rounded-md text-[10px] font-black uppercase tracking-widest hover:bg-blue-600/20 transition-all flex items-center gap-1.5"
      >
        <Plus class="w-3 h-3" /> 添加
      </button>
    </div>

    <div
      class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4"
    >
      <div
        v-for="host in hosts"
        :key="host.id"
        class="group bg-[#2d2d2d]/40 border border-white/5 rounded-2xl p-5 hover:border-white/10 transition-all cursor-pointer relative"
        @click="$emit('select-host', host.id)"
      >
        <div
          v-if="hostStats[host.id]?.current"
          :class="[
            'absolute top-4 right-4 px-2 py-0.5 rounded-full text-[10px] font-bold flex items-center gap-1',
            hostStats[host.id].current < 50
              ? 'bg-green-500/10 text-green-400'
              : 'bg-amber-500/10 text-amber-400',
          ]"
        >
          {{ hostStats[host.id].current.toFixed(0) }}ms
        </div>

        <div class="flex items-center gap-3 mb-4">
          <div
            :class="[
              'w-2.5 h-2.5 rounded-full',
              hostStats[host.id]?.status === 'Success' ||
              (hostStats[host.id]?.success_rate || 0) > 0
                ? 'bg-green-500 shadow-[0_0_8px_rgba(34,197,94,0.4)]'
                : 'bg-slate-600',
            ]"
          ></div>
          <div>
            <h3
              class="font-bold text-sm text-white group-hover:text-blue-400 transition-colors"
            >
              {{ host.name }}
            </h3>
            <p class="text-[10px] text-slate-500 font-mono mt-0.5">
              {{ host.address }}
            </p>
          </div>
        </div>

        <div class="flex items-center gap-3 pt-3 border-t border-white/5">
          <span class="text-[10px] text-slate-600"
            ><Activity class="w-3 h-3 inline mr-1" />
            {{ host.display_rules[0]?.threshold }}ms</span
          >
          <div class="flex gap-1.5 overflow-hidden">
            <span
              v-for="label in hostStats[host.id]?.labels"
              :key="label"
              class="px-2 py-0.5 bg-green-500/10 text-green-400 rounded-md text-[8px] font-bold whitespace-nowrap"
            >
              # {{ label }}
            </span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
