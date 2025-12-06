<script setup>
import { useDesktopPet } from '../composables/DesktopPet'

const {
  // 响应式状态
  isHovering,
  hasClipboardWindow,
  hasMenuWindow,
  isDragging,

  // 计算属性
  petImagePath,
  petBackgroundPath,
  showPetAnimation,

  // 事件处理函数
  handlePointerEnter,
  handlePointerLeave,
  handlePointerDown,
  handleLeftClick,
  handleContextMenu,
  animationFrame
} = useDesktopPet()


</script>

<template>
  <div
    class="desktop-pet"
    :style="{
      cursor: isDragging ? 'grabbing' : 'grab'
    }"
    @pointerenter="handlePointerEnter"
    @pointerleave="handlePointerLeave"
    @pointerdown="handlePointerDown"
    @click="handleLeftClick"
    @contextmenu="handleContextMenu"
  >
    <div class="pet-container">
      <!-- 背景层 -->
      <img
        :src="petBackgroundPath"
        alt="Pet Background"
        draggable="false"
        class="pet-background"
      />
      
      <!-- 动画层（按键动画等） -->
      <img
        v-if="showPetAnimation"
        :src="petImagePath"
        alt="Desktop Pet Animation"
        draggable="false"
        :class="['pet-animation', { 'hover': isHovering, 'has-window': hasMenuWindow }]"
      />

      <!-- live2d层 -->
      <canvas id="live2dCanvas" class="pet-live2d"></canvas>
    </div>
  </div>
</template>

<style scoped>
.desktop-pet {
  position: fixed;
  width: 150px;
  height: 150px;
  z-index: 9999;
  user-select: none;
  pointer-events: auto;
  background: transparent;
}

.pet-container {
  width: 100%;
  height: 100%;
  display: flex;
  top: 10px;
  left: 10px;
  background: transparent;
  position: relative;
}

.pet-background {
  position: absolute;
  width: 130px;
  height: 75px;
  z-index: 1; /* 背景在最底层 */
  pointer-events: none; /* 背景不接收鼠标事件 */
  object-fit: contain;
}

.pet-animation {
  position: absolute;
  width: 130px;
  height: 75px;
  z-index: 3; /* 动画层在背景之上 */
  pointer-events: auto; /* 动画层接收鼠标事件 */
  object-fit: contain;
}

.pet-live2d {
  position: absolute;
  width: 130px;
  height: 75px;
  z-index: 2; /* 动画层在背景之上 */
  pointer-events: auto; /* 动画层接收鼠标事件 */
  object-fit: contain;
}
</style>