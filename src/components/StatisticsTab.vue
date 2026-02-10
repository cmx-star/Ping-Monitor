<script setup lang="ts">
import { computed } from "vue";
import { Line } from "vue-chartjs";
import {
  Chart as ChartJS,
  CategoryScale,
  LinearScale,
  PointElement,
  LineElement,
  Title,
  Tooltip,
  Legend,
  Filler,
} from "chart.js";

ChartJS.register(
  CategoryScale,
  LinearScale,
  PointElement,
  LineElement,
  Title,
  Tooltip,
  Legend,
  Filler
);

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

const props = defineProps<{
  hosts: HostConfig[];
  hostStats: Record<string, PingStats>;
  hostHistory: Record<string, number[]>;
  selectedHostId: string | null;
}>();

const emit = defineEmits<{
  (e: "update:selectedHostId", id: string | null): void;
}>();

const currentHost = computed(() => {
  if (!props.selectedHostId) return null;
  return props.hosts.find((h) => h.id === props.selectedHostId) || null;
});

const currentStats = computed(() => {
  if (!props.selectedHostId) return null;
  return props.hostStats[props.selectedHostId] || null;
});

const aggregatedStats = computed(() => {
  const allStats = Object.values(props.hostStats);
  const total = allStats.reduce((acc, s) => acc + s.total_pings, 0);
  const success = allStats.reduce((acc, s) => acc + s.successful_pings, 0);
  const failed = allStats.reduce((acc, s) => acc + s.failed_pings, 0);
  const bytesSent = allStats.reduce((acc, s) => acc + s.bytes_sent, 0);
  const bytesReceived = allStats.reduce((acc, s) => acc + s.bytes_received, 0);

  // Find earliest start time
  const earliestStart =
    allStats.length > 0
      ? allStats.reduce(
          (min, s) => (s.start_time < min ? s.start_time : min),
          allStats[0].start_time
        )
      : new Date().toISOString();

  return {
    total_pings: total,
    successful_pings: success,
    failed_pings: failed,
    success_rate: total > 0 ? (success / total) * 100 : 0,
    packet_loss_rate: total > 0 ? (failed / total) * 100 : 0,
    online_count: allStats.filter((s) => s.success_rate > 0).length,
    total_hosts: allStats.length,
    mean:
      allStats.length > 0
        ? allStats.reduce((acc, s) => acc + s.mean, 0) / allStats.length
        : 0,
    current:
      allStats.length > 0
        ? allStats.reduce((acc, s) => acc + s.current, 0) / allStats.length
        : 0,
    bytes_sent: bytesSent,
    bytes_received: bytesReceived,
    min: allStats.length > 0 ? Math.min(...allStats.map((s) => s.min)) : 0,
    max: allStats.length > 0 ? Math.max(...allStats.map((s) => s.max)) : 0,
    start_time: earliestStart,
  };
});

const chartData = computed(() => {
  const targetId = props.selectedHostId;
  if (!targetId) return { labels: [], datasets: [] };

  const history = props.hostHistory[targetId] || [];

  return {
    labels: Array(history.length).fill(""),
    datasets: [
      {
        label: "Latency",
        borderColor: "#3b82f6",
        backgroundColor: (context: any) => {
          const ctx = context.chart.ctx;
          const gradient = ctx.createLinearGradient(0, 0, 0, 200);
          gradient.addColorStop(0, "rgba(59, 130, 246, 0.25)");
          gradient.addColorStop(1, "rgba(59, 130, 246, 0.05)");
          return gradient;
        },
        borderWidth: 2,
        pointRadius: (ctx: any) => {
          // Show dot for the last point only
          const index = ctx.dataIndex;
          const count = ctx.dataset.data.length;
          return index === count - 1 ? 4 : 0;
        },
        pointBackgroundColor: (ctx: any) => {
          const val = ctx.raw;
          if (val < 50) return "#22c55e";
          if (val < 100) return "#f97316";
          return "#ef4444";
        },
        pointBorderColor: "#ffffff",
        pointBorderWidth: 1,
        data: history,
        fill: true,
        tension: 0.1, // Straighter lines as per screenshot
      },
    ],
  };
});

const chartOptions = {
  responsive: true,
  maintainAspectRatio: false,
  plugins: { legend: { display: false }, tooltip: { enabled: true } },
  scales: {
    y: {
      beginAtZero: true,
      grid: { display: false }, // No grid lines on Y as per screenshot
      ticks: { display: false }, // No ticks on Y
    },
    x: { display: false },
  },
  animation: { duration: 0 },
  elements: {
    line: {
      tension: 0, // Straight lines
    },
  },
};

const formatBytes = (bytes: number) => {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB", "TB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
};

const formatDuration = (startTime: string) => {
  if (!startTime) return "0秒";
  const start = new Date(startTime);
  const now = new Date();
  const diff = Math.floor((now.getTime() - start.getTime()) / 1000);

  if (diff < 60) return `${diff}秒`;
  if (diff < 3600) return `${Math.floor(diff / 60)}分${diff % 60}秒`;
  return `${Math.floor(diff / 3600)}小时${Math.floor((diff % 3600) / 60)}分`;
};
</script>

<template>
  <div class="space-y-6">
    <!-- Header Selection -->
    <div
      class="flex items-center justify-between gap-4 bg-[#2d2d2d]/30 p-4 rounded-xl border border-white/5"
    >
      <div class="flex items-center gap-4">
        <select
          :value="selectedHostId"
          @change="emit('update:selectedHostId', ($event.target as HTMLSelectElement).value || null)"
          class="bg-[#1e1e1e] border border-white/10 text-white rounded-md px-4 py-1.5 text-xs font-bold outline-none accent-blue-500 min-w-[200px]"
        >
          <option :value="''">全部主机 (聚合)</option>
          <option v-for="h in hosts" :key="h.id" :value="h.id">
            {{ h.name }}
          </option>
        </select>
        <div v-if="currentHost" class="flex flex-col">
          <span class="text-xs font-bold text-white">{{
            currentHost.name
          }}</span>
          <span class="text-[10px] text-slate-500 font-mono">{{
            currentHost.address
          }}</span>
        </div>
      </div>
    </div>

    <!-- Row 1: Overview Cards (4 cols) -->
    <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
      <!-- Total Pings -->
      <div
        class="bg-[#2d2d2d]/30 p-5 rounded-xl border border-white/5 flex flex-col items-center justify-center gap-2"
      >
        <div class="text-blue-500 text-2xl">
          <!-- Icon: number.circle.fill equivalent -->
          <svg
            xmlns="http://www.w3.org/2000/svg"
            width="24"
            height="24"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            class="lucide lucide-hash"
          >
            <line x1="4" x2="20" y1="9" y2="9" />
            <line x1="4" x2="20" y1="15" y2="15" />
            <line x1="10" x2="8" y1="3" y2="21" />
            <line x1="16" x2="14" y1="3" y2="21" />
          </svg>
        </div>
        <div class="text-xl font-bold font-mono text-white">
          {{
            selectedHostId
              ? currentStats?.total_pings ?? 0
              : aggregatedStats.total_pings
          }}
        </div>
        <div class="text-[10px] text-slate-500 uppercase font-bold">请求数</div>
      </div>

      <!-- Success Rate -->
      <div
        class="bg-[#2d2d2d]/30 p-5 rounded-xl border border-white/5 flex flex-col items-center justify-center gap-2"
      >
        <div class="text-green-500 text-2xl">
          <!-- Icon: checkmark.circle.fill -->
          <svg
            xmlns="http://www.w3.org/2000/svg"
            width="24"
            height="24"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            class="lucide lucide-check-circle"
          >
            <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14" />
            <path d="m9 11 3 3L22 4" />
          </svg>
        </div>
        <div class="text-xl font-bold font-mono text-white">
          {{
            (selectedHostId
              ? currentStats?.success_rate ?? 0
              : aggregatedStats.success_rate
            ).toFixed(1)
          }}%
        </div>
        <div class="text-[10px] text-slate-500 uppercase font-bold">成功率</div>
      </div>

      <!-- Packet Loss -->
      <div
        class="bg-[#2d2d2d]/30 p-5 rounded-xl border border-white/5 flex flex-col items-center justify-center gap-2"
      >
        <div
          :class="[
            'text-2xl',
            (selectedHostId
              ? currentStats?.packet_loss_rate ?? 0
              : aggregatedStats.packet_loss_rate) > 5
              ? 'text-red-500'
              : 'text-orange-500',
          ]"
        >
          <!-- Icon: xmark.circle.fill -->
          <svg
            xmlns="http://www.w3.org/2000/svg"
            width="24"
            height="24"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            class="lucide lucide-x-circle"
          >
            <circle cx="12" cy="12" r="10" />
            <path d="m15 9-6 6" />
            <path d="m9 9 6 6" />
          </svg>
        </div>
        <div class="text-xl font-bold font-mono text-white">
          {{
            (selectedHostId
              ? currentStats?.packet_loss_rate ?? 0
              : aggregatedStats.packet_loss_rate
            ).toFixed(1)
          }}%
        </div>
        <div class="text-[10px] text-slate-500 uppercase font-bold">丢包率</div>
      </div>

      <!-- Total Traffic -->
      <div
        class="bg-[#2d2d2d]/30 p-5 rounded-xl border border-white/5 flex flex-col items-center justify-center gap-2"
      >
        <div class="text-purple-500 text-2xl">
          <!-- Icon: arrow.up.arrow.down.circle.fill -->
          <svg
            xmlns="http://www.w3.org/2000/svg"
            width="24"
            height="24"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            class="lucide lucide-arrow-up-down"
          >
            <path d="m21 16-4 4-4-4" />
            <path d="M17 20V4" />
            <path d="m3 8 4-4 4 4" />
            <path d="M7 4v16" />
          </svg>
        </div>
        <div class="text-xl font-bold font-mono text-white">
          {{
            formatBytes(
              (selectedHostId
                ? currentStats?.bytes_sent ?? 0
                : aggregatedStats.bytes_sent) +
                (selectedHostId
                  ? currentStats?.bytes_received ?? 0
                  : aggregatedStats.bytes_received)
            )
          }}
          <span class="text-xs text-slate-500 ml-1 font-normal">总流量</span>
        </div>
        <div class="text-[10px] text-slate-500 uppercase font-bold">总流量</div>
      </div>
    </div>

    <!-- Row 2: Latency Chart -->
    <div class="bg-[#2d2d2d]/30 p-4 rounded-xl border border-white/5">
      <div class="flex items-center justify-between mb-4">
        <h3 class="text-sm font-semibold text-slate-300">延迟趋势</h3>

        <!-- Legend -->
        <div class="flex items-center gap-4 text-[10px]">
          <div class="flex items-center gap-1.5">
            <div class="w-1.5 h-1.5 rounded-full bg-green-500"></div>
            <span class="text-slate-500">优秀</span>
          </div>
          <div class="flex items-center gap-1.5">
            <div class="w-1.5 h-1.5 rounded-full bg-orange-500"></div>
            <span class="text-slate-500">良好</span>
          </div>
          <div class="flex items-center gap-1.5">
            <div class="w-1.5 h-1.5 rounded-full bg-red-500"></div>
            <span class="text-slate-500">较差</span>
          </div>
        </div>
      </div>

      <div class="h-[180px] relative">
        <Line
          v-if="selectedHostId && chartData.datasets[0].data.length > 0"
          :data="chartData"
          :options="chartOptions"
        />
        <div
          v-else
          class="h-full flex items-center justify-center text-slate-600 text-xs italic"
        >
          {{ selectedHostId ? "等待数据中..." : "聚合模式下暂不支持趋势图表" }}
        </div>
        <!-- "Last N times" indicator simulation -->
        <div
          v-if="selectedHostId"
          class="absolute top-0 right-0 text-[10px] text-slate-500"
        >
          最近 100 次
        </div>
      </div>
    </div>

    <!-- Row 3: Detailed Stats -->
    <div class="bg-[#2d2d2d]/30 p-4 rounded-xl border border-white/5">
      <h3 class="text-sm font-semibold text-slate-300 mb-4">
        {{
          selectedHostId
            ? "详细统计"
            : `详细统计 (${aggregatedStats.total_hosts || hosts.length} 个主机)`
        }}
      </h3>

      <div class="grid grid-cols-2 gap-x-8 gap-y-3 mb-6">
        <!-- Col 1 -->
        <div class="flex justify-between items-center py-1">
          <span class="text-xs text-slate-500">成功请求</span>
          <span class="text-xs font-mono font-bold text-white">{{
            selectedHostId
              ? currentStats?.successful_pings ?? 0
              : aggregatedStats.successful_pings
          }}</span>
        </div>
        <div class="flex justify-between items-center py-1">
          <span class="text-xs text-slate-500">失败请求</span>
          <span class="text-xs font-mono font-bold text-white">{{
            selectedHostId
              ? currentStats?.failed_pings ?? 0
              : aggregatedStats.failed_pings
          }}</span>
        </div>

        <div class="flex justify-between items-center py-1">
          <span class="text-xs text-slate-500">最小延迟</span>
          <span class="text-xs font-mono font-bold text-white">
            {{
              (selectedHostId
                ? currentStats?.min ?? 0
                : aggregatedStats.min) === Number.POSITIVE_INFINITY ||
              (selectedHostId
                ? currentStats?.min ?? 0
                : aggregatedStats.min) === 0
                ? "N/A"
                : (selectedHostId
                    ? currentStats?.min ?? 0
                    : aggregatedStats.min
                  ).toFixed(2) + " ms"
            }}
          </span>
        </div>
        <div class="flex justify-between items-center py-1">
          <span class="text-xs text-slate-500">最大延迟</span>
          <span class="text-xs font-mono font-bold text-white">
            {{
              (selectedHostId
                ? currentStats?.max ?? 0
                : aggregatedStats.max) === Number.NEGATIVE_INFINITY ||
              (selectedHostId
                ? currentStats?.max ?? 0
                : aggregatedStats.max) === 0
                ? "N/A"
                : (selectedHostId
                    ? currentStats?.max ?? 0
                    : aggregatedStats.max
                  ).toFixed(2) + " ms"
            }}
          </span>
        </div>

        <div class="flex justify-between items-center py-1">
          <span class="text-xs text-slate-500">平均延迟</span>
          <span class="text-xs font-mono font-bold text-white">
            {{
              (selectedHostId
                ? currentStats?.mean ?? 0
                : aggregatedStats.mean
              ).toFixed(2)
            }}
            ms
          </span>
        </div>
        <div class="flex justify-between items-center py-1">
          <span class="text-xs text-slate-500">运行时间</span>
          <span class="text-xs font-mono font-bold text-white">
            {{
              formatDuration(
                selectedHostId
                  ? currentStats?.start_time ?? ""
                  : aggregatedStats.start_time
              )
            }}
          </span>
        </div>
      </div>

      <!-- Traffic Footer -->
      <div
        class="flex justify-between items-center border-t border-white/5 pt-4"
      >
        <div class="flex flex-col gap-1">
          <span class="text-[10px] text-slate-500">发送流量</span>
          <span class="text-sm font-mono text-slate-300">{{
            formatBytes(
              selectedHostId
                ? currentStats?.bytes_sent ?? 0
                : aggregatedStats.bytes_sent
            )
          }}</span>
        </div>
        <div class="flex flex-col items-end gap-1">
          <span class="text-[10px] text-slate-500">接收流量</span>
          <span class="text-sm font-mono text-slate-300">{{
            formatBytes(
              selectedHostId
                ? currentStats?.bytes_received ?? 0
                : aggregatedStats.bytes_received
            )
          }}</span>
        </div>
      </div>
    </div>
  </div>
</template>
