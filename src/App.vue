<!-- App.vue -->
<template>
  <router-view></router-view>
</template>

<script setup>
// 这里可以放一些全局的逻辑
</script>

<style>
/* 全局样式可以放在这里 */
/* 全局样式 - 确保桌宠应用透明无边框 */
html, body {
  margin: 0;
  padding: 0;
  background: transparent;
  overflow: hidden;
  width: 100vw;
  height: 100vh;
}

#app {
  width: 100%;
  height: 100%;
  background: transparent;
}
</style>