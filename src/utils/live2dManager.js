import { convertFileSrc } from '@tauri-apps/api/core'
import { readDir, readTextFile } from '@tauri-apps/plugin-fs'
import { Cubism4ModelSettings, Live2DModel } from 'pixi-live2d-display'
import { Application, Ticker } from 'pixi.js'

// 全局注册Ticker
Live2DModel.registerTicker(Ticker)

class Live2dManager {
  app = null
  model = null
  modelSettings = null

  constructor() {
    console.log('Live2dManager 初始化')
  }

  // 初始化PIXI应用 - 只在这里禁用交互
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

    // 创建PIXI应用，禁用交互以避免兼容性问题
    this.app = new Application({
      view: canvas,
      resizeTo: window,
      backgroundAlpha: 0,
      resolution: devicePixelRatio,
      autoStart: true,
      autoDensity: true,
      // 关键：禁用交互
      interaction: false
    })

    console.log('PIXI 应用创建成功')
  }

  // 加载模型 - 使用之前成功的方法
  async load(path) {
    try {
      console.log('=== 开始加载模型 ===')
      console.log('模型路径:', path)
      
      // 首先尝试读取目录内容
      console.log('尝试读取目录内容...')
      const files = await readDir(path)
      console.log(`目录包含 ${files.length} 个文件`)
      
      // 查找模型文件
      const modelFile = files.find(file => 
        file.name.endsWith('.model3.json') || 
        file.name.endsWith('.model.json')
      )
      
      if (!modelFile) {
        throw new Error('未找到模型文件')
      }

      console.log('模型文件:', modelFile.name)
      const modelPath = `${path}/${modelFile.name}`
      console.log('完整模型路径:', modelPath)

      // 读取模型配置文件
      console.log('读取模型配置文件...')
      const modelContent = await readTextFile(modelPath)
      const modelJSON = JSON.parse(modelContent)
      console.log('模型配置解析成功')

      // 创建模型设置
      console.log('创建模型设置...')
      this.modelSettings = new Cubism4ModelSettings({
        ...modelJSON,
        url: convertFileSrc(modelPath),
      })

      console.log('模型设置创建成功')

      // 处理资源文件路径 - 使用之前成功的方法
      console.log('处理资源文件路径...')
      
      this.modelSettings.replaceFiles((file) => {
        // 构建完整路径
        let fullPath
        if (file.includes('/')) {
          // 处理子目录路径（如 demomodel.1024/texture_00.png）
          fullPath = modelPath.substring(0, modelPath.lastIndexOf('/')) + '/' + file
        } else {
          // 处理相对路径
          fullPath = modelPath.substring(0, modelPath.lastIndexOf('/')) + '/' + file
        }
        
        // 转换为URL
        const url = convertFileSrc(fullPath)
        console.log(`转换路径: ${file} -> ${fullPath} -> ${url}`)
        return url
      })

      // 确保PIXI应用已初始化
      this.initApp()
      
      // 清理现有模型
      if (this.model) {
        this.destroy()
      }

      // 加载Live2D模型
      console.log('创建Live2D模型实例...')
      this.model = await Live2DModel.from(this.modelSettings)
      
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

      console.log('模型加载完成:', {
        width: this.model.width,
        height: this.model.height
      })

      console.log('=== 模型加载成功 ===')

      return {
        width: this.model.width,
        height: this.model.height,
        motions: this.modelSettings.motions || {},
        expressions: this.modelSettings.expressions || [],
        model: this.model
      }

    } catch (error) {
      console.error('加载模型时出错:', error)
      
      // 提供详细的错误信息
      if (error.message.includes('500')) {
        console.error('500错误：可能的原因：')
        console.error('1. Tauri asset协议配置问题')
        console.error('2. 文件路径错误')
        console.error('3. 文件不存在')
        
        // 打印更多调试信息
        console.log('当前路径:', path)
        console.log('尝试检查模型目录内容...')
        try {
          const files = await readDir(path)
          console.log('目录内容:', files.map(f => f.name))
        } catch (e) {
          console.error('无法读取目录:', e)
        }
      }
      
      throw error
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
    
    // 如果容器大小为0，使用默认值
    if (containerWidth === 0 || containerHeight === 0) {
      console.warn('容器大小为零，使用默认值')
      return
    }
    
    const scaleX = containerWidth / this.model.width
    const scaleY = containerHeight / this.model.height
    const scale = Math.min(scaleX, scaleY)

    this.model.scale.set(scale)
    //this.model.x = this.app.screen.width / 2
    //this.model.y = this.app.screen.height / 2
    this.model.x = 60
    this.model.y = 37
    console.log('model.x', this.model.x, 'model.y', this.model.y)
    console.log('app.screen.width', this.app.screen.width, 'app.screen.width', this.app.screen.height)
    
    console.log('模型已调整大小:', { 
      scale, 
      containerWidth, 
      containerHeight,
      modelWidth: this.model.width,
      modelHeight: this.model.height
    })
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
}

// 创建单例实例
const live2d = new Live2dManager()
export default live2d