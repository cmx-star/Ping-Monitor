<script setup lang="ts">
import { computed } from "vue";

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

const props = defineProps<{
  show: boolean;
  isEdit: boolean;
  modelValue: Partial<HostConfig>;
}>();

const emit = defineEmits<{
  (e: "update:modelValue", value: Partial<HostConfig>): void;
  (e: "close"): void;
  (e: "confirm"): void;
}>();

const localHost = computed({
  get: () => props.modelValue,
  set: (val) => emit("update:modelValue", val),
});

const handleAddressChange = (e: Event) => {
  const val = (e.target as HTMLInputElement).value;
  // Simple auto-name if empty
  if (!localHost.value.name && val) {
    localHost.value.name = val;
  }
};
</script>

<template>
  <Transition name="modal">
    <div
      v-if="show"
      class="fixed inset-0 z-50 flex items-center justify-center p-6 bg-black/80 backdrop-blur-sm"
    >
      <div
        class="w-full max-w-md bg-[#2d2d2d] border border-white/10 rounded-2xl p-8 shadow-2xl"
      >
        <h2 class="text-lg font-bold text-white mb-6">
          {{ isEdit ? "修改主机配置" : "添加新的监控主机" }}
        </h2>
        <div class="space-y-5">
          <div class="space-y-1.5">
            <label class="text-[10px] font-bold text-slate-500 uppercase"
              >标识名称</label
            >
            <input
              v-model="localHost.name"
              placeholder="例如: 腾讯云香港"
              class="w-full bg-[#1e1e1e] border border-white/10 rounded-lg px-4 py-2 text-sm text-white focus:border-blue-500 outline-none"
            />
          </div>
          <div class="space-y-1.5">
            <label class="text-[10px] font-bold text-slate-500 uppercase"
              >IP 地址 / 域名</label
            >
            <input
              v-model="localHost.address"
              @input="handleAddressChange"
              placeholder="hk.tencent.com"
              class="w-full bg-[#1e1e1e] border border-white/10 rounded-lg px-4 py-2 text-sm text-white font-mono focus:border-blue-500 outline-none"
            />
          </div>
          <!-- Custom Command (Optional) -->
          <div class="space-y-1.5">
            <label class="text-[10px] font-bold text-slate-500 uppercase"
              >自定义 Ping 命令 (可选)</label
            >
            <input
              v-model="localHost.command"
              placeholder="默认: ping -i 1 $address"
              class="w-full bg-[#1e1e1e] border border-white/10 rounded-lg px-4 py-2 text-sm text-white font-mono focus:border-blue-500 outline-none"
            />
          </div>

          <div class="flex gap-3 pt-4">
            <button
              @click="$emit('close')"
              class="flex-1 px-4 py-2 bg-white/5 hover:bg-white/10 text-white rounded-lg text-sm font-bold transition-all border border-white/5"
            >
              取消
            </button>
            <button
              @click="$emit('confirm')"
              class="flex-1 px-4 py-2 bg-blue-600 hover:bg-blue-500 text-white rounded-lg text-sm font-bold transition-all shadow-xl shadow-blue-600/20"
            >
              {{ isEdit ? "更新配置" : "开始监控" }}
            </button>
          </div>
        </div>
      </div>
    </div>
  </Transition>
</template>

<style scoped>
.modal-enter-active,
.modal-leave-active {
  transition: all 0.3s cubic-bezier(0.16, 1, 0.3, 1);
}

.modal-enter-from,
.modal-leave-to {
  opacity: 0;
  transform: scale(0.95) translateY(10px);
}
</style>
