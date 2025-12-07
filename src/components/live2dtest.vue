<template>
  <div class="live2d-container">
    <!-- 主控制面板 -->
    <div class="main-controls">
      <button @click="toggleDebugPanel">调试面板</button>
      <button @click="changeExpression">换表情</button>
      <button @click="toggleMotion">切换动作</button>
      <button @click="resetModel">重置</button>
      <button @click="printParameters">打印参数</button>
    </div>
    
    <!-- 参数调试面板 -->
    <div v-if="showDebugPanel" class="debug-panel">
      <div class="debug-header">
        <h3>参数调试</h3>
        <div class="debug-controls">
          <button @click="resetAllParameters">重置所有参数</button>
          <button @click="savePreset">保存预设</button>
          <button @click="loadPreset">加载预设</button>
          <button @click="copyAllValues">复制数值</button>
          <button @click="saveParameterState">保存状态</button>
          <button @click="loadParameterState">加载状态</button>
        </div>
      </div>
      
      <!-- 参数统计 -->
      <div class="parameter-stats" v-if="parameterStats">
        共 {{ parameterStats.total }} 个参数，已修改 {{ parameterStats.changed }} 个
      </div>
      
      <div class="parameter-groups">
        <div v-for="group in parameterGroups" :key="group.id" class="parameter-group">
          <div class="group-header" @click="toggleGroup(group.id)">
            <span>{{ group.name }} ({{ group.parameters.length }})</span>
            <span>{{ expandedGroups[group.id] ? '▲' : '▼' }}</span>
          </div>
          
          <div v-if="expandedGroups[group.id]" class="group-parameters">
            <div v-for="param in group.parameters" :key="param.id" class="parameter-item">
              <div class="parameter-info">
                <span class="param-name">{{ param.name }}</span>
                <span class="param-id">{{ param.id }}</span>
                <span class="param-value">{{ getParameterValue(param.id).toFixed(3) }}</span>
              </div>
              
              <div class="parameter-controls">
                <input 
                  type="range" 
                  :min="param.min || -1" 
                  :max="param.max || 1" 
                  :step="0.01"
                  :value="getParameterValue(param.id)"
                  @input="updateParameterValue(param.id, $event.target.valueAsNumber)"
                  class="param-slider"
                />
                
                <input 
                  type="number" 
                  :min="param.min || -1" 
                  :max="param.max || 1" 
                  :step="0.01"
                  :value="getParameterValue(param.id)"
                  @input="updateParameterValue(param.id, parseFloat($event.target.value) || 0)"
                  class="param-input"
                />
                
                <button @click="resetSingleParameter(param.id)" class="param-reset">重置</button>
                
                <!-- 添加相对调整按钮 -->
                <button @click="addParameterValue(param.id, 0.1)" class="param-add">+0.1</button>
                <button @click="addParameterValue(param.id, -0.1)" class="param-subtract">-0.1</button>
              </div>
            </div>
          </div>
        </div>
      </div>
      
      <!-- 快速调整区域 -->
      <div class="quick-adjust">
        <h4>快速调整</h4>
        <div class="quick-controls">
          <div class="quick-group">
            <label>眼睛开闭:</label>
            <input type="range" min="0" max="1" step="0.01" 
                   :value="getParameterValue('ParamEyeLOpen')"
                   @input="updateEyeOpen($event.target.valueAsNumber)" />
            <span>{{ getParameterValue('ParamEyeLOpen').toFixed(2) }}</span>
          </div>
          
          <div class="quick-group">
            <label>嘴巴开合:</label>
            <input type="range" min="0" max="1" step="0.01" 
                   :value="getParameterValue('ParamMouthOpenY')"
                   @input="updateParameterValue('ParamMouthOpenY', $event.target.valueAsNumber)" />
            <span>{{ getParameterValue('ParamMouthOpenY').toFixed(2) }}</span>
          </div>
          
          <div class="quick-group">
            <label>头部角度X:</label>
            <input type="range" min="-30" max="30" step="0.1" 
                   :value="getParameterValue('ParamAngleX') * 30"
                   @input="updateParameterValue('ParamAngleX', $event.target.valueAsNumber / 30)" />
            <span>{{ (getParameterValue('ParamAngleX') * 30).toFixed(1) }}°</span>
          </div>
          
          <div class="quick-group">
            <label>头部角度Y:</label>
            <input type="range" min="-30" max="30" step="0.1" 
                   :value="getParameterValue('ParamAngleY') * 30"
                   @input="updateParameterValue('ParamAngleY', $event.target.valueAsNumber / 30)" />
            <span>{{ (getParameterValue('ParamAngleY') * 30).toFixed(1) }}°</span>
          </div>
        </div>
        
        <!-- 操作按钮 -->
        <div class="action-buttons">
          <button @click="saveParameters()">保存参数状态</button>
          <button @click="loadParameters()">加载参数状态</button>
          <button @click="exportParameters()">导出配置</button>
          <button @click="importParameters()">导入配置</button>
        </div>
      </div>
    </div>
    
    <!-- 状态信息 -->
    <div class="status" v-if="loading">
      加载中...
    </div>
    <div class="error" v-if="error">
      {{ error }}
    </div>
    <div class="info" v-if="info">
      {{ info }}
    </div>
  </div>
</template>

<script setup>
import { onMounted, onUnmounted, ref, reactive, computed } from 'vue'
import live2d from '../utils/live2dManager.js'

// 状态
const showDebugPanel = ref(false)
const modelInfo = ref(null)
const motions = ref([])
const expressions = ref([])
const parameters = ref([])
const parameterGroups = ref([])
const loading = ref(false)
const error = ref(null)
const info = ref(null)
const expandedGroups = reactive({})
const presets = ref(JSON.parse(localStorage.getItem('live2d_presets') || '{}'))

// 计算参数统计
const parameterStats = computed(() => {
  if (!parameters.value.length) return null
  
  const total = parameters.value.length
  const changed = parameters.value.filter(param => {
    const currentValue = getParameterValue(param.id)
    return Math.abs(currentValue - param.defaultValue) > 0.001
  }).length
  
  return {
    total,
    changed,
    unchanged: total - changed
  }
})

// 加载模型
const loadModel = async () => {
  try {
    console.log('开始加载模型...')
    loading.value = true
    error.value = null
    
    // 使用绝对路径 - 之前成功过的路径
    const modelPath = 'C:/Users/heyufei/Desktop/bigHW/SmartPaste/public/resources/live2d'
    console.log('使用路径:', modelPath)
    info.value = '正在加载模型...'
    
    const result = await live2d.load(modelPath)
    
    modelInfo.value = result
    motions.value = result.motions ? Object.keys(result.motions) : []
    expressions.value = result.expressions || []
    parameters.value = result.parameters || []
    parameterGroups.value = result.parameterGroups || []
    
    // 默认展开所有分组
    parameterGroups.value.forEach(group => {
      expandedGroups[group.id] = true
    })
    
    // 初始调整大小
    setTimeout(() => {
      live2d.resizeModel()
      info.value = `模型加载成功！共 ${parameters.value.length} 个参数`
      loading.value = false
    }, 100)
    
    console.log('模型加载成功', result)
  } catch (err) {
    console.error('加载模型失败:', err)
    error.value = err.message
    loading.value = false
    info.value = '加载失败'
  }
}

// 获取参数值 - 使用正确的方法名
const getParameterValue = (id) => {
  return live2d.getParameterValue(id) || 0
}

// 更新单个参数 - 使用正确的方法名
const updateParameterValue = (id, value) => {
  const success = live2d.setParameterValue(id, value)
  if (success) {
    // 延迟更新信息，避免频繁触发
    setTimeout(() => {
      info.value = `${id}: ${value.toFixed(3)}`
    }, 50)
  }
}

// 相对调整参数值
const addParameterValue = (id, value) => {
  const success = live2d.addParameterValue(id, value)
  if (success) {
    info.value = `${id} 调整了 ${value > 0 ? '+' : ''}${value.toFixed(3)}`
  }
}

// 更新双眼开闭
const updateEyeOpen = (value) => {
  live2d.setParameterValue('ParamEyeLOpen', value)
  live2d.setParameterValue('ParamEyeROpen', value)
  info.value = `双眼: ${value.toFixed(3)}`
}

// 重置单个参数 - 使用正确的方法名
const resetSingleParameter = (id) => {
  const success = live2d.resetParameter(id)
  if (success) {
    const param = parameters.value.find(p => p.id === id)
    const defaultValue = param ? param.defaultValue : 0
    info.value = `${id} 已重置为 ${defaultValue.toFixed(3)}`
  }
}

// 重置所有参数 - 使用正确的方法名
const resetAllParameters = () => {
  if (live2d.resetParameters()) {
    info.value = '所有参数已重置'
  }
}

// 保存参数状态
const saveParameterState = () => {
  if (live2d.saveParameters()) {
    info.value = '参数状态已保存'
  }
}

// 加载参数状态
const loadParameterState = () => {
  if (live2d.loadParameters()) {
    info.value = '参数状态已加载'
  }
}

// 导出参数配置
const exportParameters = () => {
  const config = live2d.exportParameters()
  const configStr = JSON.stringify(config, null, 2)
  
  // 创建下载链接
  const blob = new Blob([configStr], { type: 'application/json' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = `live2d-config-${new Date().toISOString().slice(0, 10)}.json`
  document.body.appendChild(a)
  a.click()
  document.body.removeChild(a)
  URL.revokeObjectURL(url)
  
  info.value = '参数配置已导出'
}

// 导入参数配置
const importParameters = () => {
  const input = document.createElement('input')
  input.type = 'file'
  input.accept = '.json'
  input.onchange = (e) => {
    const file = e.target.files[0]
    if (!file) return
    
    const reader = new FileReader()
    reader.onload = (event) => {
      try {
        const config = JSON.parse(event.target.result)
        if (live2d.importParameters(config)) {
          info.value = '参数配置已导入'
        }
      } catch (err) {
        error.value = '配置文件格式错误'
      }
    }
    reader.readAsText(file)
  }
  input.click()
}

// 打印参数信息
const printParameters = () => {
  live2d.printParameters()
  info.value = '参数信息已打印到控制台'
}

// 切换分组展开状态
const toggleGroup = (groupId) => {
  expandedGroups[groupId] = !expandedGroups[groupId]
}

// 切换调试面板
const toggleDebugPanel = () => {
  showDebugPanel.value = !showDebugPanel.value
  info.value = showDebugPanel.value ? '调试面板已打开' : '调试面板已关闭'
}

// 保存预设
const savePreset = () => {
  const presetName = prompt('请输入预设名称:')
  if (presetName) {
    const config = live2d.exportParameters()
    config.name = presetName
    
    presets.value[presetName] = config
    localStorage.setItem('live2d_presets', JSON.stringify(presets.value))
    info.value = `预设 "${presetName}" 已保存`
  }
}

// 加载预设
const loadPreset = () => {
  const presetNames = Object.keys(presets.value)
  if (presetNames.length === 0) {
    info.value = '没有可用的预设'
    return
  }
  
  const selected = prompt(`选择预设: ${presetNames.join(', ')}`)
  if (selected && presets.value[selected]) {
    const preset = presets.value[selected]
    if (live2d.importParameters(preset)) {
      info.value = `预设 "${selected}" 已加载`
    }
  }
}

// 复制所有数值
const copyAllValues = () => {
  const values = parameters.value.map(param => {
    return `${param.name} (${param.id}): ${getParameterValue(param.id).toFixed(3)}`
  }).join('\n')
  
  navigator.clipboard.writeText(values)
    .then(() => {
      info.value = '所有参数值已复制到剪贴板'
    })
    .catch(err => {
      console.error('复制失败:', err)
      info.value = '复制失败'
    })
}

// 切换动作
const toggleMotion = () => {
  if (!motions.value.length) {
    console.warn('没有可用动作')
    info.value = '没有可用动作'
    return
  }
  
  // 获取第一个动作组
  const motionGroup = motions.value[0]
  if (motionGroup && modelInfo.value?.motions?.[motionGroup]) {
    const motionList = modelInfo.value.motions[motionGroup]
    if (motionList && motionList.length > 0) {
      const randomIndex = Math.floor(Math.random() * motionList.length)
      live2d.playMotion(motionGroup, randomIndex)
      info.value = `播放动作: ${motionGroup}[${randomIndex}]`
    }
  }
}

// 切换表情
const changeExpression = () => {
  if (!expressions.value.length) {
    console.warn('没有可用表情')
    info.value = '没有可用表情'
    return
  }
  
  const randomIndex = Math.floor(Math.random() * expressions.value.length)
  live2d.playExpressions(randomIndex)
  info.value = `播放表情: ${randomIndex}`
}

// 重置模型
const resetModel = () => {
  info.value = '重置模型中...'
  loadModel()
}

// 窗口大小变化处理
const handleResize = () => {
  if (modelInfo.value) {
    setTimeout(() => {
      live2d.resizeModel()
    }, 100)
  }
}

// 组件挂载
onMounted(async () => {
  console.log('Live2D组件挂载')
  
  // 监听窗口大小变化
  window.addEventListener('resize', handleResize)
  
  // 延迟加载模型，确保DOM已渲染
  setTimeout(async () => {
    await loadModel()
  }, 1000)
})

// 组件卸载
onUnmounted(() => {
  console.log('Live2D组件卸载')
  window.removeEventListener('resize', handleResize)
  
  // 清理资源
  try {
    live2d.destroy()
  } catch (err) {
    console.warn('清理资源时出错:', err)
  }
})
</script>

<style scoped>
.live2d-container {
  position: relative;
  width: 100%;
  height: 800px; /* 增加高度以容纳调试面板 */
  overflow: hidden;
  background: #f5f5f5;
  border-radius: 10px;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.1);
}

.main-controls {
  position: absolute;
  top: 20px;
  left: 50%;
  transform: translateX(-50%);
  display: flex;
  gap: 10px;
  background: rgba(255, 255, 255, 0.95);
  padding: 15px 25px;
  border-radius: 30px;
  box-shadow: 0 4px 15px rgba(0, 0, 0, 0.15);
  z-index: 10;
  backdrop-filter: blur(10px);
  border: 1px solid rgba(255, 255, 255, 0.2);
}

.main-controls button {
  padding: 10px 20px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  border: none;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.3s ease;
  font-size: 14px;
  font-weight: 500;
  letter-spacing: 0.5px;
  box-shadow: 0 4px 15px rgba(102, 126, 234, 0.4);
}

.main-controls button:hover {
  transform: translateY(-2px);
  box-shadow: 0 6px 20px rgba(102, 126, 234, 0.6);
}

/* 调试面板样式 */
.debug-panel {
  position: absolute;
  top: 80px;
  left: 20px;
  right: 20px;
  background: rgba(255, 255, 255, 0.98);
  border-radius: 10px;
  padding: 20px;
  box-shadow: 0 8px 30px rgba(0, 0, 0, 0.2);
  z-index: 20;
  max-height: 600px;
  overflow-y: auto;
  backdrop-filter: blur(20px);
  border: 1px solid rgba(255, 255, 255, 0.3);
}

.debug-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
  padding-bottom: 15px;
  border-bottom: 2px solid #eaeaea;
}

.debug-header h3 {
  margin: 0;
  color: #333;
  font-size: 18px;
  font-weight: 600;
}

.debug-controls {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.debug-controls button {
  padding: 6px 12px;
  background: linear-gradient(135deg, #4CAF50 0%, #45a049 100%);
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 12px;
  font-weight: 500;
  transition: all 0.3s ease;
  white-space: nowrap;
}

.debug-controls button:hover {
  transform: translateY(-1px);
  box-shadow: 0 4px 8px rgba(76, 175, 80, 0.3);
}

.debug-controls button:nth-child(2) {
  background: linear-gradient(135deg, #2196F3 0%, #1976D2 100%);
}

.debug-controls button:nth-child(3) {
  background: linear-gradient(135deg, #FF9800 0%, #F57C00 100%);
}

.debug-controls button:nth-child(4) {
  background: linear-gradient(135deg, #9C27B0 0%, #7B1FA2 100%);
}

.debug-controls button:nth-child(5) {
  background: linear-gradient(135deg, #00bcd4 0%, #0097a7 100%);
}

.debug-controls button:nth-child(6) {
  background: linear-gradient(135deg, #795548 0%, #5d4037 100%);
}

/* 参数统计样式 */
.parameter-stats {
  background: #e8f5e9;
  padding: 8px 12px;
  border-radius: 6px;
  margin-bottom: 15px;
  font-size: 14px;
  color: #2e7d32;
  border: 1px solid #c8e6c9;
}

/* 参数分组样式 */
.parameter-groups {
  margin-bottom: 20px;
}

.parameter-group {
  margin-bottom: 15px;
  border: 1px solid #e0e0e0;
  border-radius: 8px;
  overflow: hidden;
  background: #f9f9f9;
}

.group-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 15px;
  background: linear-gradient(135deg, #6a11cb 0%, #2575fc 100%);
  color: white;
  cursor: pointer;
  font-weight: 500;
  user-select: none;
  transition: all 0.3s ease;
}

.group-header:hover {
  background: linear-gradient(135deg, #5a0cb9 0%, #1c65e8 100%);
}

.group-parameters {
  padding: 15px;
}

.parameter-item {
  margin-bottom: 12px;
  padding: 12px;
  background: white;
  border-radius: 6px;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.05);
}

.parameter-info {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 10px;
}

.param-name {
  font-weight: 500;
  color: #333;
  flex: 2;
  font-size: 14px;
}

.param-id {
  font-size: 11px;
  color: #666;
  font-family: monospace;
  flex: 1;
  text-align: center;
}

.param-value {
  font-family: monospace;
  font-weight: 600;
  color: #2196F3;
  flex: 1;
  text-align: right;
  font-size: 14px;
}

.parameter-controls {
  display: flex;
  gap: 8px;
  align-items: center;
  flex-wrap: wrap;
}

.param-slider {
  flex: 3;
  min-width: 150px;
  height: 8px;
  border-radius: 4px;
  background: linear-gradient(to right, #e0e0e0, #4CAF50);
  outline: none;
  -webkit-appearance: none;
}

.param-slider::-webkit-slider-thumb {
  -webkit-appearance: none;
  width: 20px;
  height: 20px;
  border-radius: 50%;
  background: white;
  border: 2px solid #4CAF50;
  cursor: pointer;
  box-shadow: 0 2px 6px rgba(0, 0, 0, 0.2);
}

.param-input {
  flex: 1;
  min-width: 80px;
  padding: 6px;
  border: 1px solid #ddd;
  border-radius: 4px;
  text-align: center;
  font-family: monospace;
  font-size: 12px;
}

.param-reset {
  padding: 6px 10px;
  background: #ff9800;
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 12px;
  transition: all 0.3s ease;
  white-space: nowrap;
}

.param-reset:hover {
  background: #f57c00;
  transform: translateY(-1px);
}

.param-add, .param-subtract {
  padding: 6px 10px;
  background: #9c27b0;
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 12px;
  transition: all 0.3s ease;
  white-space: nowrap;
}

.param-add:hover {
  background: #7b1fa2;
  transform: translateY(-1px);
}

.param-subtract {
  background: #673ab7;
}

.param-subtract:hover {
  background: #512da8;
  transform: translateY(-1px);
}

/* 快速调整区域 */
.quick-adjust {
  margin-top: 20px;
  padding: 15px;
  background: #f0f7ff;
  border-radius: 8px;
  border: 1px solid #bbdefb;
}

.quick-adjust h4 {
  margin: 0 0 15px 0;
  color: #1976D2;
  font-size: 16px;
}

.quick-controls {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 15px;
  margin-bottom: 15px;
}

.quick-group {
  display: flex;
  flex-direction: column;
  gap: 5px;
}

.quick-group label {
  font-size: 12px;
  color: #555;
  font-weight: 500;
}

.quick-group input[type="range"] {
  width: 100%;
  height: 6px;
}

.quick-group span {
  font-size: 12px;
  color: #2196F3;
  font-weight: 600;
  font-family: monospace;
}

.action-buttons {
  display: flex;
  gap: 10px;
  flex-wrap: wrap;
}

.action-buttons button {
  padding: 8px 12px;
  background: linear-gradient(135deg, #00bcd4 0%, #0097a7 100%);
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 12px;
  transition: all 0.3s ease;
}

.action-buttons button:hover {
  transform: translateY(-1px);
  box-shadow: 0 4px 8px rgba(0, 188, 212, 0.3);
}

.action-buttons button:nth-child(2) {
  background: linear-gradient(135deg, #795548 0%, #5d4037 100%);
}

.action-buttons button:nth-child(3) {
  background: linear-gradient(135deg, #607d8b 0%, #455a64 100%);
}

.action-buttons button:nth-child(4) {
  background: linear-gradient(135deg, #8bc34a 0%, #689f38 100%);
}

.status {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  background: rgba(0, 0, 0, 0.7);
  color: white;
  padding: 20px 40px;
  border-radius: 10px;
  font-size: 18px;
  z-index: 5;
}

.error {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  background: rgba(231, 76, 60, 0.9);
  color: white;
  padding: 20px 40px;
  border-radius: 10px;
  font-size: 16px;
  text-align: center;
  max-width: 80%;
  white-space: pre-line;
  z-index: 5;
}

.info {
  position: absolute;
  top: 70px;
  left: 50%;
  transform: translateX(-50%);
  background: rgba(52, 152, 219, 0.9);
  color: white;
  padding: 10px 20px;
  border-radius: 5px;
  font-size: 14px;
  text-align: center;
  max-width: 90%;
  z-index: 5;
}

#live2dCanvas {
  display: block;
  width: 100%;
  height: 100%;
}

/* 滚动条样式 */
.debug-panel::-webkit-scrollbar {
  width: 8px;
}

.debug-panel::-webkit-scrollbar-track {
  background: #f1f1f1;
  border-radius: 4px;
}

.debug-panel::-webkit-scrollbar-thumb {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  border-radius: 4px;
}

.debug-panel::-webkit-scrollbar-thumb:hover {
  background: linear-gradient(135deg, #5a6fd8 0%, #6a4190 100%);
}

/* 响应式调整 */
@media (max-width: 1200px) {
  .parameter-controls {
    flex-direction: column;
    align-items: stretch;
  }
  
  .param-slider {
    min-width: 100%;
  }
  
  .quick-controls {
    grid-template-columns: 1fr;
  }
}
</style>