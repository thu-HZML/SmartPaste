import { convertFileSrc } from '@tauri-apps/api/core'
import { readDir, readTextFile } from '@tauri-apps/plugin-fs'
import { Cubism4ModelSettings, Live2DModel } from 'pixi-live2d-display'
import { Application, Ticker } from 'pixi.js'

Live2DModel.registerTicker(Ticker)

class Live2dManager {
  app = null
  model = null
  parameters = []  // 存储参数信息
  parameterGroups = []  // 存储参数分组
  parameterIndexMap = {}  // 参数ID到索引的映射

  constructor() {
    console.log('Live2dManager 初始化')
  }

  // 初始化PIXI应用
  initApp() {
    if (this.app) return

    console.log('初始化 PIXI 应用...')
    
    // 创建canvas元素
    let canvas = document.getElementById('live2dCanvas')
    if (!canvas) {
      canvas = document.createElement('canvas')
      canvas.id = 'live2dCanvas'
      canvas.style.width = '100%'
      canvas.style.height = '100%'
      canvas.style.display = 'block'
      
      // 添加到页面
      const container = document.querySelector('.live2d-container')
      if (container) {
        container.appendChild(canvas)
      } else {
        console.error('找不到 .live2d-container 元素')
        return
      }
    }

    // 创建PIXI应用
    this.app = new Application({
      view: canvas,
      resizeTo: window,
      backgroundAlpha: 0,
      resolution: devicePixelRatio,
      autoStart: true,
      autoDensity: true,
      interaction: false
    })

    console.log('PIXI 应用创建成功')
  }

  // 加载模型
  async load(path) {
    try {
      console.log('=== 开始加载模型 ===')
      console.log('模型路径:', path)
      
      // 读取目录内容
      const files = await readDir(path)
      console.log(`目录包含 ${files.length} 个文件`)
      
      // 查找模型文件
      const modelFile = files.find(file => 
        file.name.endsWith('.model3.json') || 
        file.name.endsWith('.model.json')
      )
      
      if (!modelFile) {
        throw new Error('未找到模型主配置文件')
      }

      console.log('模型文件:', modelFile.name)
      const modelPath = `${path}/${modelFile.name}`
      
      // 读取模型配置文件
      const modelContent = await readTextFile(modelPath)
      const modelJSON = JSON.parse(modelContent)

      // 创建模型设置
      const modelSettings = new Cubism4ModelSettings({
        ...modelJSON,
        url: convertFileSrc(modelPath),
      })

      // 处理资源文件路径
      modelSettings.replaceFiles((file) => {
        let fullPath
        if (file.includes('/')) {
          fullPath = modelPath.substring(0, modelPath.lastIndexOf('/')) + '/' + file
        } else {
          fullPath = modelPath.substring(0, modelPath.lastIndexOf('/')) + '/' + file
        }
        return convertFileSrc(fullPath)
      })

      // 初始化PIXI应用
      this.initApp()
      
      // 清理现有模型
      this.destroy()

      // 加载Live2D模型
      this.model = await Live2DModel.from(modelSettings)
      
      if (!this.model) {
        throw new Error('创建Live2D模型失败')
      }

      // 禁用模型交互
      this.model.interactive = false
      this.model.interactiveChildren = false
      
      // 添加到舞台
      this.app.stage.addChild(this.model)
      
      // 居中显示
      this.model.x = this.app.screen.width / 2
      this.model.y = this.app.screen.height / 2
      this.model.anchor.set(0.5)

      // 加载参数信息
      await this.loadParameterInfo(path)

      console.log('=== 模型加载成功 ===')

      return {
        width: this.model.width,
        height: this.model.height,
        motions: modelSettings.motions || {},
        expressions: modelSettings.expressions || [],
        model: this.model,
        parameters: this.parameters,
        parameterGroups: this.parameterGroups
      }

    } catch (error) {
      console.error('加载模型时出错:', error)
      throw error
    }
  }

  // 加载参数信息
  async loadParameterInfo(modelPath) {
    try {
      // 读取CDI文件
      const cdiPath = `${modelPath}/demomodel.cdi3.json`
      const cdiContent = await readTextFile(cdiPath)
      const cdiJSON = JSON.parse(cdiContent)
      
      // 提取参数信息
      this.parameters = []
      cdiJSON.Parameters.forEach(param => {
        const paramInfo = this.getParameterInfo(param.Id)
        this.parameters.push({
          id: param.Id,
          name: param.Name,
          groupId: param.GroupId || '',
          ...paramInfo
        })
      })
      
      // 提取分组信息
      this.parameterGroups = cdiJSON.ParameterGroups.map(group => ({
        id: group.Id,
        name: group.Name,
        parameters: this.parameters.filter(p => p.groupId === group.Id)
      }))
      
      // 为没有分组的参数创建一个默认分组
      const ungrouped = this.parameters.filter(p => !p.groupId)
      if (ungrouped.length > 0) {
        this.parameterGroups.unshift({
          id: 'ungrouped',
          name: '未分组参数',
          parameters: ungrouped
        })
      }
      
      console.log(`加载了 ${this.parameters.length} 个参数，${this.parameterGroups.length} 个分组`)
      
    } catch (error) {
      console.warn('无法加载CDI文件，将从coreModel获取参数信息:', error)
      this.loadParametersFromCoreModel()
    }
  }

  // 获取参数信息（包括当前值、默认值和范围）
  getParameterInfo(id) {
    const coreModel = this.getCoreModel()
    if (!coreModel) {
      return { min: 0, max: 1, value: 0, defaultValue: 0 }
    }

    try {
      // 获取参数索引
      const index = coreModel.getParameterIndex(id)
      if (index === undefined || index === -1) {
        console.warn(`参数 ${id} 不存在`)
        return { min: 0, max: 1, value: 0, defaultValue: 0 }
      }

      // 缓存索引映射
      this.parameterIndexMap[id] = index
      
      // 获取参数范围、默认值和当前值
      const min = coreModel.getParameterMinimumValue(index)
      const max = coreModel.getParameterMaximumValue(index)
      const defaultValue = coreModel.getParameterDefaultValue(index)
      const value = coreModel.getParameterValueById(id) // 使用正确的方法获取当前值
      
      return {
        min: min !== undefined ? min : 0,
        max: max !== undefined ? max : 1,
        defaultValue: defaultValue !== undefined ? defaultValue : 0,
        value: value !== undefined ? value : 0
      }
    } catch (error) {
      console.error(`获取参数 ${id} 信息时出错:`, error)
      return { min: 0, max: 1, value: 0, defaultValue: 0 }
    }
  }

  // 从coreModel获取所有参数
  loadParametersFromCoreModel() {
    try {
      const coreModel = this.getCoreModel()
      if (!coreModel) {
        console.warn('无法获取coreModel')
        return
      }

      // 获取参数数量
      const parameterCount = coreModel.getParameterCount()
      console.log(`coreModel中有 ${parameterCount} 个参数`)

      // 遍历所有参数
      this.parameters = []
      for (let i = 0; i < parameterCount; i++) {
        try {
          const id = coreModel.getParameterId(i)
          if (id) {
            const paramInfo = this.getParameterInfo(id)
            this.parameters.push({
              id: id,
              name: id,
              groupId: '',
              ...paramInfo
            })
          }
        } catch (e) {
          console.warn(`获取参数 ${i} 时出错:`, e.message)
        }
      }

      // 创建一个默认分组
      this.parameterGroups = [{
        id: 'default',
        name: '参数',
        parameters: this.parameters
      }]

      console.log(`从coreModel加载了 ${this.parameters.length} 个参数`)

    } catch (error) {
      console.error('从coreModel加载参数失败:', error)
    }
  }

  // 获取coreModel
  getCoreModel() {
    if (!this.model || !this.model.internalModel) {
      console.warn('模型未加载或没有internalModel')
      return null
    }
    return this.model.internalModel.coreModel
  }

  // 获取参数值 - 使用正确的方法
  getParameterValue(id) {
    const coreModel = this.getCoreModel()
    if (!coreModel) return 0

    try {
      // 方法1: 直接使用getParameterValueById
      const value = coreModel.getParameterValueById(id)
      if (value !== undefined) {
        return value
      }
      
      // 方法2: 如果上面失败，尝试通过索引获取
      const index = coreModel.getParameterIndex(id)
      if (index !== undefined && index !== -1) {
        return coreModel.getParameterValueByIndex(index) || 0
      }
      
      console.warn(`参数 ${id} 不存在`)
      return 0
    } catch (error) {
      console.error(`获取参数 ${id} 值时出错:`, error)
      return 0
    }
  }

  // 设置参数值 - 使用正确的方法
  setParameterValue(id, value) {
    const coreModel = this.getCoreModel()
    if (!coreModel) return false

    try {
      // 获取参数信息以限制范围
      const param = this.parameters.find(p => p.id === id)
      if (!param) {
        console.warn(`参数 ${id} 不存在`)
        return false
      }
      
      // 限制值在有效范围内
      const clampedValue = Math.max(param.min, Math.min(param.max, value))
      
      // 使用setParameterValueById设置参数值
      coreModel.setParameterValueById(id, clampedValue)
      
      // 更新参数缓存
      param.value = clampedValue
      
      console.log(`设置参数 ${id} = ${clampedValue.toFixed(3)}`)
      return true
      
    } catch (error) {
      console.error(`设置参数 ${id} 值时出错:`, error)
      return false
    }
  }

  // 添加参数值（相对变化）
  addParameterValue(id, value) {
    const coreModel = this.getCoreModel()
    if (!coreModel) return false

    try {
      // 使用addParameterValueById进行相对调整
      coreModel.addParameterValueById(id, value)
      
      // 更新参数缓存
      const param = this.parameters.find(p => p.id === id)
      if (param) {
        param.value = this.getParameterValue(id)
      }
      
      console.log(`参数 ${id} 增加了 ${value.toFixed(3)}`)
      return true
      
    } catch (error) {
      console.error(`增加参数 ${id} 值时出错:`, error)
      return false
    }
  }

  // 乘以参数值（比例变化）
  multiplyParameterValue(id, value) {
    const coreModel = this.getCoreModel()
    if (!coreModel) return false

    try {
      // 使用multiplyParameterValueById进行比例调整
      coreModel.multiplyParameterValueById(id, value)
      
      // 更新参数缓存
      const param = this.parameters.find(p => p.id === id)
      if (param) {
        param.value = this.getParameterValue(id)
      }
      
      console.log(`参数 ${id} 乘以了 ${value.toFixed(3)}`)
      return true
      
    } catch (error) {
      console.error(`乘以参数 ${id} 值时出错:`, error)
      return false
    }
  }

  // 批量设置参数
  setParameters(parameters) {
    const coreModel = this.getCoreModel()
    if (!coreModel) return false

    try {
      let updated = false
      parameters.forEach(param => {
        if (this.setParameterValue(param.id, param.value)) {
          updated = true
        }
      })
      return updated
    } catch (error) {
      console.error('批量设置参数时出错:', error)
      return false
    }
  }

  // 重置参数到默认值
  resetParameters() {
    if (!this.model) return false
    
    try {
      this.parameters.forEach(param => {
        if (param.defaultValue !== undefined) {
          this.setParameterValue(param.id, param.defaultValue)
        }
      })
      return true
    } catch (error) {
      console.error('重置参数时出错:', error)
      return false
    }
  }

  // 重置单个参数
  resetParameter(id) {
    const param = this.parameters.find(p => p.id === id)
    if (param && param.defaultValue !== undefined) {
      return this.setParameterValue(id, param.defaultValue)
    }
    return false
  }

  // 获取所有参数当前值
  getAllParameterValues() {
    return this.parameters.map(param => ({
      ...param,
      value: this.getParameterValue(param.id)
    }))
  }

  // 保存当前参数状态
  saveParameters() {
    const coreModel = this.getCoreModel()
    if (!coreModel) return false
    
    try {
      coreModel.saveParameters()
      console.log('参数状态已保存')
      return true
    } catch (error) {
      console.error('保存参数状态时出错:', error)
      return false
    }
  }

  // 加载参数状态
  loadParameters() {
    const coreModel = this.getCoreModel()
    if (!coreModel) return false
    
    try {
      coreModel.loadParameters()
      console.log('参数状态已加载')
      return true
    } catch (error) {
      console.error('加载参数状态时出错:', error)
      return false
    }
  }

  // 销毁模型
  destroy() {
    if (this.model) {
      try {
        if (this.app?.stage) {
          this.app.stage.removeChild(this.model)
        }
        this.model.destroy({ children: true, texture: true, baseTexture: true })
        this.model = null
        this.parameters = []
        this.parameterGroups = []
        this.parameterIndexMap = {}
        console.log('模型已销毁')
      } catch (error) {
        console.warn('销毁模型时出错:', error)
      }
    }
  }

  // 调整模型大小
  resizeModel() {
    if (!this.model || !this.app) {
      console.warn('模型或应用未初始化')
      return
    }

    const container = document.getElementById('live2dCanvas')
    if (!container) {
      console.warn('找不到容器元素')
      return
    }

    const containerWidth = container.clientWidth
    const containerHeight = container.clientHeight
    
    if (containerWidth === 0 || containerHeight === 0) {
      console.warn('容器大小为零')
      return
    }
    
    const scaleX = containerWidth / this.model.width
    const scaleY = containerHeight / this.model.height
    const scale = Math.min(scaleX, scaleY)

    this.model.scale.set(scale)
    this.model.x = 60
    this.model.y = 37
    
    console.log('模型已调整大小')
  }

  // 播放动作
  playMotion(group, index = 0) {
    if (!this.model) {
      console.warn('模型未加载')
      return null
    }
    
    try {
      console.log(`播放动作: ${group}[${index}]`)
      return this.model.motion(group, index)
    } catch (error) {
      console.error('播放动作时出错:', error)
      return null
    }
  }

  // 播放表情
  playExpressions(index = 0) {
    if (!this.model) {
      console.warn('模型未加载')
      return null
    }
    
    try {
      console.log(`播放表情: ${index}`)
      return this.model.expression(index)
    } catch (error) {
      console.error('播放表情时出错:', error)
      return null
    }
  }

  // 导出当前参数配置
  exportParameters() {
    const config = {
      parameters: {},
      timestamp: new Date().toISOString()
    }
    
    this.parameters.forEach(param => {
      config.parameters[param.id] = {
        name: param.name,
        value: this.getParameterValue(param.id),
        min: param.min,
        max: param.max,
        defaultValue: param.defaultValue
      }
    })
    
    return config
  }

  // 导入参数配置
  importParameters(config) {
    if (!config || !config.parameters) return false
    
    try {
      Object.entries(config.parameters).forEach(([id, data]) => {
        if (data.value !== undefined) {
          this.setParameterValue(id, data.value)
        }
      })
      return true
    } catch (error) {
      console.error('导入参数配置时出错:', error)
      return false
    }
  }

  // 获取参数统计信息
  getParameterStats() {
    const total = this.parameters.length
    const changed = this.parameters.filter(p => 
      Math.abs(p.value - p.defaultValue) > 0.001
    ).length
    
    return {
      total,
      changed,
      unchanged: total - changed
    }
  }

  // 打印参数信息（调试用）
  printParameters() {
    console.log('=== 参数信息 ===')
    this.parameters.forEach(param => {
      console.log(`${param.name} (${param.id}): ${this.getParameterValue(param.id).toFixed(3)} [${param.min.toFixed(2)}, ${param.max.toFixed(2)}]`)
    })
  }
}

// 创建单例实例
const live2d = new Live2dManager()
export default live2d