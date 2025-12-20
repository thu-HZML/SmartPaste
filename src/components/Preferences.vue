<template>
  <div class="settings-container">
    <div class="settings-content">
      <!-- 左侧导航栏 -->
      <nav class="settings-nav">
        <ul class="nav-list">
          <li 
            v-for="item in navItems" 
            :key="item.id"
            :class="['nav-item', { active: activeNav === item.id }]"
            @click="setActiveNav(item.id)"
          >
            <component :is="item.icon" class="nav-icon" />
            <span class="nav-text">{{ item.name }}</span>
          </li>
        </ul>
      </nav>

      <!-- 右侧设置面板 -->
      <div class="settings-panel">
        <!-- 通用设置 -->
        <div v-if="activeNav === 'general'" class="panel-section">
          <h2>通用设置</h2>
          
          <div class="setting-item">
            <div class="setting-info">
              <h3>启动时自动运行</h3>
              <p>开机时自动启动剪贴板管理器</p>
            </div>
            <div class="setting-control">
              <label class="toggle-switch">
                <input 
                  type="checkbox" 
                  :checked="settings.autostart" 
                  @change="updateSetting('autostart', $event.target.checked)"
                >
                <span class="slider"></span>
              </label>
            </div>
          </div>

          <div class="setting-item">
            <div class="setting-info">
              <h3>显示系统托盘图标</h3>
              <p>在系统托盘显示应用图标</p>
            </div>
            <div class="setting-control">
              <label class="toggle-switch">
                <input 
                  type="checkbox" 
                  :checked="settings.tray_icon_visible" 
                  @change="updateSetting('tray_icon_visible', $event.target.checked)"
                >
                <span class="slider"></span>
              </label>
            </div>
          </div>
          
          <div class="setting-item">
            <div class="setting-info">
              <h3>历史记录保留时间</h3>
              <p>自动删除超过指定天数的历史记录</p>
            </div>
            <div class="setting-control">
              <select 
                v-model="settings.retention_days" 
                @change="updateSetting('retention_days', Number($event.target.value))" 
                class="select-input"
              >
                <option value=7>7天</option>
                <option value=30>30天</option>
                <option value=90>90天</option>
                <option value=0>永久保存</option>
              </select>
            </div>
          </div>
        </div>

        <!-- 快捷键设置 -->
        <div v-if="activeNav === 'shortcuts'" class="panel-section">
          <h2>快捷键设置</h2>
          
          <div class="hint">
            <p>提示：点击快捷键输入框，然后按下您想要设置的组合键</p>
            <p>按 ESC 键可取消设置</p>
          </div>

          <div v-for="key in shortcutKeys" :key="key" class="setting-item">
            <div class="setting-info">
              <h3>{{ shortcutDisplayNames[key] }}</h3>
              <p>自定义全局快捷键</p>
            </div>
            <div class="setting-control">
              <input 
                type="text" 
                :value="settings[key]" 
                :class="['shortcut-input', { 'recording-active': shortcutManager.isRecording && shortcutManager.currentType === key }]"
                @click="startRecording(key)"
                readonly
                :placeholder="shortcutManager.isRecording && shortcutManager.currentType === key ? '正在录制...' : '点击设置'"
              >
            </div>
          </div>
          
        </div>

        <!-- 剪贴板参数设置 -->
        <div v-if="activeNav === 'clipboard'" class="panel-section">
          <h2>剪贴板参数设置</h2>
          
          <div class="setting-item">
            <div class="setting-info">
              <h3>最大历史记录数量</h3>
              <p>限制保存的剪贴板历史记录数量</p>
            </div>
            <div class="setting-control">
              <input 
                type="number" 
                v-model="settings.max_history_items" 
                min="10" 
                max="1000" 
                class="number-input"
                @change="updateSetting('max_history_items', Number($event.target.value))" 
              >
            </div>
          </div>
          
          <div class="setting-item">
            <div class="setting-info">
              <h3>忽略短文本</h3>
              <p>不保存字符数少于指定值的文本</p>
            </div>
            <div class="setting-control">
              <input 
                type="number" 
                v-model="settings.ignore_short_text_len" 
                min="0" 
                max="50" 
                class="number-input"
                @change="updateSetting('ignore_short_text_len', Number($event.target.value))" 
              >
              <span class="unit">字符</span>
            </div>
          </div>

          <div class="setting-item">
            <div class="setting-info">
              <h3>忽略大文件</h3>
              <p>不保存字符数大于指定值的文件</p>
            </div>
            <div class="setting-control">
              <input 
                type="number" 
                v-model="settings.ignore_big_file_mb" 
                min="5" 
                max="100" 
                class="number-input"
                @change="updateSetting('ignore_big_file_mb', Number($event.target.value))"
              >
              <span class="unit">MB</span>
            </div>
          </div>

          <div class="setting-item">
            <div class="setting-info">
              <h3>删除确认</h3>
              <p>删除剪贴板内容时弹出确认对话框</p>
            </div>
            <div class="setting-control">
              <label class="toggle-switch">
                <input 
                  type="checkbox" 
                  :checked="settings.delete_confirmation" 
                  @change="updateSetting('delete_confirmation', $event.target.checked)"
                >
                <span class="slider"></span>
              </label>
            </div>
          </div>

          <div class="setting-item">
            <div class="setting-info">
              <h3>收藏保留</h3>
              <p>点击全部删除按钮时是否保留已收藏内容</p>
            </div>
            <div class="setting-control">
              <label class="toggle-switch">
                <input 
                  type="checkbox" 
                  :checked="settings.keep_favorites_on_delete" 
                  @change="updateSetting('keep_favorites_on_delete', $event.target.checked)"
                >
                <span class="slider"></span>
              </label>
            </div>
          </div>

          <div class="setting-item">
            <div class="setting-info">
              <h3>自动排序</h3>
              <p>复制已存在的内容时排列到最前面</p>
            </div>
            <div class="setting-control">
              <label class="toggle-switch">
                <input 
                  type="checkbox" 
                  :checked="settings.auto_sort" 
                  @change="updateSetting('auto_sort', $event.target.checked)"
                >
                <span class="slider"></span>
              </label>
            </div>
          </div>


        </div>

        <!-- OCR设置 -->
        <div v-if="activeNav === 'ocr'" class="panel-section">
          <h2>OCR设置</h2>

          <div class="setting-item">
            <div class="setting-info">
              <h3>OCR提供者</h3>
              <p>选择OCR识别服务提供者</p>
            </div>
            <div class="setting-control">
              <select 
                v-model="settings.ocr_provider" 
                @change="updateSetting('ocr_provider', $event.target.value)" 
                class="select-input"
              >
                <option value="auto">默认</option>
                <option value="tesseract">Tesseract</option>
                <option value="windows">Windows OCR</option>
                <option value="baidu">百度OCR</option>
                <option value="google">Google Vision</option>
                <option value="custom">自定义</option>
              </select>
            </div>
          </div>

          <div class="setting-item">
            <div class="setting-info">
              <h3>识别语言</h3>
              <p>选择OCR识别的语言，支持多语言同时识别</p>
            </div>
            <div class="setting-control">
              <div class="checkbox-group">
                <label class="checkbox-item">
                  <input 
                    type="checkbox" 
                    :checked="settings.ocr_languages && settings.ocr_languages.includes('chi_sim')" 
                    @change="toggleOCRLanguage('chi_sim', $event.target.checked)"
                  > 简体中文
                </label>
                <label class="checkbox-item">
                  <input 
                    type="checkbox" 
                    :checked="settings.ocr_languages && settings.ocr_languages.includes('eng')" 
                    @change="toggleOCRLanguage('eng', $event.target.checked)"
                  > 英语
                </label>
                <label class="checkbox-item">
                  <input 
                    type="checkbox" 
                    :checked="settings.ocr_languages && settings.ocr_languages.includes('jpn')" 
                    @change="toggleOCRLanguage('jpn', $event.target.checked)"
                  > 日语
                </label>
                <label class="checkbox-item">
                  <input 
                    type="checkbox" 
                    :checked="settings.ocr_languages && settings.ocr_languages.includes('kor')" 
                    @change="toggleOCRLanguage('kor', $event.target.checked)"
                  > 韩语
                </label>
                <label class="checkbox-item">
                  <input 
                    type="checkbox" 
                    :checked="settings.ocr_languages && settings.ocr_languages.includes('fra')" 
                    @change="toggleOCRLanguage('fra', $event.target.checked)"
                  > 法语
                </label>
                <label class="checkbox-item">
                  <input 
                    type="checkbox" 
                    :checked="settings.ocr_languages && settings.ocr_languages.includes('deu')" 
                    @change="toggleOCRLanguage('deu', $event.target.checked)"
                  > 德语
                </label>
              </div>
            </div>
          </div>

          <div class="setting-item">
            <div class="setting-info">
              <h3>置信度阈值</h3>
              <p>设置识别结果的置信度阈值，低于此值的结果将被忽略</p>
            </div>
            <div class="setting-control">
              <div class="slider-container">
                <input 
                  type="range" 
                  :value="settings.ocr_confidence_threshold" 
                  min="0" 
                  max="100" 
                  step="1" 
                  class="slider-input"
                  @input="updateSetting('ocr_confidence_threshold', Number($event.target.value))"
                >
                <span class="slider-value">{{ settings.ocr_confidence_threshold }}%</span>
              </div>
            </div>
          </div>

          <div class="setting-item">
            <div class="setting-info">
              <h3>超时时间</h3>
              <p>设置OCR识别的最长等待时间（秒）</p>
            </div>
            <div class="setting-control">
              <input 
                type="number" 
                v-model="settings.ocr_timeout_secs" 
                min="5" 
                max="120" 
                class="number-input"
                @change="updateSetting('ocr_timeout_secs', Number($event.target.value))"
              >
              <span class="unit">秒</span>
            </div>
          </div>
        </div>

        <!-- AI Agent 设置 -->
        <div v-if="activeNav === 'ai'" class="panel-section">
          <h2>AI Agent 设置</h2>

          <div class="setting-item">
            <div class="setting-info">
              <h3>启用AI助手</h3>
              <p>启用AI智能助手功能</p>
            </div>
            <div class="setting-control">
              <label class="toggle-switch">
                <input 
                  type="checkbox" 
                  :checked="settings.ai_enabled" 
                  @change="updateSetting('ai_enabled', $event.target.checked)"
                >
                <span class="slider"></span>
              </label>
            </div>
          </div>

          <div v-if="settings.ai_enabled" class="ai-settings">
            <div class="setting-item">
              <div class="setting-info">
                <h3>选择AI服务</h3>
                <p>选择使用的AI服务提供商</p>
              </div>
              <div class="setting-control">
                <select 
                  v-model="settings.ai_provider" 
                  @change="updateSetting('ai_provider', $event.target.value)" 
                  class="select-input"
                >
                  <option value="default">默认</option>
                  <option value="openai">OpenAI</option>
                  <option value="google">Google</option>
                  <option value="aliyun">Aliyun</option>
                  <option value="deepseek">DeepSeek</option>
                  <option value="moonshot">Moonshot</option>
                  <option value="custom">自定义</option>
                </select>
              </div>
            </div>

            <div v-if="settings.ai_provider !== 'default'" class="setting-item">
              <div class="setting-info">
                <h3>API密钥</h3>
                <p>设置AI服务的API密钥</p>
              </div>
              <div class="setting-control">
                <input 
                  type="password" 
                  v-model="settings.ai_api_key" 
                  @blur="updateSetting('ai_api_key', $event.target.value)"
                  class="text-input" 
                  placeholder="输入API密钥"
                >
              </div>
            </div>

            <div v-if="settings.ai_provider !== 'default'" class="setting-item">
              <div class="setting-info">
                <h3>base_url</h3>
                <p>设置AI服务的基础URL，如(https://llmapi.paratera.com/v1)</p>
              </div>
              <div class="setting-control">
                <input 
                  type="text" 
                  v-model="settings.ai_base_url" 
                  @blur="updateSetting('ai_base_url', $event.target.value)"
                  class="text-input" 
                  placeholder="输入base_url"
                >
              </div>
            </div>

            <div v-if="settings.ai_provider !== 'default'" class="setting-item">
              <div class="setting-info">
                <h3>模型名称</h3>
                <p>设置AI服务的模型</p>
              </div>
              <div class="setting-control">
                <input 
                  type="text" 
                  v-model="settings.ai_model" 
                  @blur="updateSetting('ai_model', $event.target.value)"
                  class="text-input" 
                  placeholder="输入模型名称"
                >
              </div>
            </div>

            <div v-if="settings.ai_provider === 'default'" class="setting-item">
              <div class="setting-info">
                <h3>选择AI模型</h3>
                <p>选择使用的AI模型</p>
              </div>
              <div class="setting-control">
                <select 
                  v-model="settings.ai_model" 
                  @change="updateSetting('ai_model', $event.target.value)" 
                  class="select-input"
                >
                  <option value="DeepSeek-V3.2">DeepSeek-V3.2</option>
                  <option value="Doubao-Seedream-4.0">Doubao-Seedream-4.0</option>
                  <option value="Qwen3-VL-235B-A22B-Instruct">Qwen3-VL-235B-A22B-Instruct</option>
                  <option value="Kimi-K2">Kimi-K2</option>
                  <option value="GLM-4.6">GLM-4.6</option>
                </select>
              </div>
            </div>

            <div class="setting-item">
              <div class="setting-info">
                <h3>采样温度</h3>
                <p>采样温度越高，ai生成文本的随机性和多样性越强</p>
              </div>
              <div class="setting-control">
                <div class="slider-container">
                  <input 
                    type="range" 
                    :value="settings.ai_temperature" 
                    min="0.5" 
                    max="2" 
                    step="0.1" 
                    class="slider-input"
                    @input="updateSetting('ai_temperature', Number($event.target.value))"
                  >
                  <span class="slider-value">{{ settings.ai_temperature }}</span>
                </div>
              </div>
            </div>

            <div class="setting-item">
              <div class="setting-info">
                <h3>AI功能开关</h3>
                <p>启用或禁用各项AI功能</p>
              </div>
              <div class="setting-control">
                <div class="checkbox-group">
                  <label class="checkbox-item">
                    <input 
                      type="checkbox" 
                      :checked="settings.ai_auto_tag" 
                      @change="updateSetting('ai_auto_tag', $event.target.checked)"
                    > 自动打Tag
                  </label>
                  <label class="checkbox-item">
                    <input 
                      type="checkbox" 
                      :checked="settings.ai_auto_summary" 
                      @change="updateSetting('ai_auto_summary', $event.target.checked)"
                    > 自动总结
                  </label>
                  <label class="checkbox-item">
                    <input 
                      type="checkbox" 
                      :checked="settings.ai_translation" 
                      @change="updateSetting('ai_translation', $event.target.checked)"
                    > 翻译
                  </label>
                  <label class="checkbox-item">
                    <input 
                      type="checkbox" 
                      :checked="settings.ai_web_search" 
                      @change="updateSetting('ai_web_search', $event.target.checked)"
                    > 联网搜索
                  </label>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- 安全与隐私 -->
        <div v-if="activeNav === 'security'" class="panel-section">
          <h2>安全与隐私</h2>

          <div class="setting-item">
            <div class="setting-info">
              <h3>敏感词过滤</h3>
              <p>自动屏蔽密码、银行卡号等敏感信息</p>
            </div>
            <div class="setting-control">
              <label class="toggle-switch">
                <input 
                  type="checkbox" 
                  :checked="settings.sensitive_filter" 
                  @change="updateSetting('sensitive_filter', $event.target.checked)"
                >
                <span class="slider"></span>
              </label>
            </div>
          </div>

          <div v-if="settings.sensitive_filter" class="setting-item">
            <div class="setting-info">
              <h3>过滤类型</h3>
              <p>选择要过滤的敏感信息类型</p>
            </div>
            <div class="setting-control">
              <div class="checkbox-group">
                <label class="checkbox-item">
                  <input 
                    type="checkbox" 
                    :checked="settings.filter_passwords" 
                    @change="updateSetting('filter_passwords', $event.target.checked)"
                  > 密码<span class="tip-text">（匹配备注中的‘密码’字样）</span>
                </label>
                <label class="checkbox-item">
                  <input 
                    type="checkbox" 
                    :checked="settings.filter_bank_cards" 
                    @change="updateSetting('filter_bank_cards', $event.target.checked)"
                  > 银行卡号
                </label>
                <label class="checkbox-item">
                  <input 
                    type="checkbox" 
                    :checked="settings.filter_id_cards" 
                    @change="updateSetting('filter_id_cards', $event.target.checked)"
                  > 身份证号
                </label>
                <label class="checkbox-item">
                  <input 
                    type="checkbox" 
                    :checked="settings.filter_phone_numbers" 
                    @change="updateSetting('filter_phone_numbers', $event.target.checked)"
                  > 手机号
                </label>
              </div>
            </div>
          </div>

          <div class="setting-item">
            <div class="setting-info">
              <h3>隐私记录管理</h3>
              <p>查看和管理标记为隐私的记录</p>
            </div>
            <div class="setting-control">
              <button class="btn btn-secondary" @click="showPrivate">查看隐私记录</button>
            </div>
          </div>
        </div>

        <!-- 数据备份 -->
        <div v-if="activeNav === 'backup'" class="panel-section">
          <h2>数据备份</h2>

          <div class="setting-item">
            <div class="setting-info">
              <h3>数据存储路径</h3>
              <p>设置数据文件的存储位置</p>
            </div>
            <div class="setting-control">
              <div class="path-input-container">
                <div class="path-input-group">
                  <input 
                    type="text" 
                    :value="settings.storage_path" 
                    class="text-input path-input" 
                    readonly
                    :title="settings.storage_path || '未设置存储路径'"
                    placeholder="点击右侧按钮选择路径"
                  >
                  <button class="btn btn-secondary path-btn" @click="changeStoragePath">
                    {{ settings.storage_path ? '更改路径' : '选择路径' }}
                  </button>
                </div>
                <div v-if="!settings.storage_path" class="path-hint">
                  <small>请选择数据存储路径</small>
                </div>
              </div>
            </div>
          </div>

          <div class="backup-actions">
            <h3>数据操作</h3>

            <div class="action-group">
              <div class="action-item">
                <div class="action-info">
                  <h4>导出数据</h4>
                  <p>将数据导出为本地文件（离线操作）</p>
                </div>
                <button class="btn btn-primary" @click="exportData">导出数据</button>
              </div>

              <div class="action-item">
                <div class="action-info">
                  <h4>导入数据</h4>
                  <p>从本地文件导入数据（离线操作）</p>
                </div>
                <button class="btn btn-secondary" @click="importData">导入数据</button>
              </div>

            </div>
          </div>
        </div>

        <!-- 云端入口 -->
        <div v-if="activeNav === 'cloud'" class="panel-section">
          <h2>云端同步</h2>
          
          <!-- 同步状态显示 -->
          <div class="sync-status" v-if="userLoggedIn">
            <div class="status-item">
              <span class="status-label">同步状态:</span>
              <span class="status-value" :class="{'success': lastSyncStatus === 'success', 'error': lastSyncStatus === 'error'}">
                {{ lastSyncStatus === 'success' ? '同步成功' : lastSyncStatus === 'error' ? '同步失败' : '未同步' }}
              </span>
            </div>
            <div class="status-item">
              <span class="status-label">上次同步时间:</span>
              <span class="status-value">
                {{ lastSyncTime ? formatTime(lastSyncTime) : '从未同步' }}
              </span>
            </div>
            <div class="status-actions">
              <button class="btn btn-small" @click="handleCloudPush" :disabled="isSyncing">
                {{ isSyncing ? '同步中...' : '立即同步' }}
              </button>
            </div>
          </div>

          <div class="setting-item">
            <div class="setting-info">
              <h3>启用云端同步</h3>
              <p>将剪贴板历史同步到云端，跨设备访问</p>
            </div>
            <div class="setting-control">
              <label class="toggle-switch">
                <input 
                  type="checkbox" 
                  :checked="settings.cloud_sync_enabled" 
                  @change="updateSetting('cloud_sync_enabled', $event.target.checked)"
                >
                <span class="slider"></span>
              </label>
            </div>
          </div>
          
          <div v-if="settings.cloud_sync_enabled" class="cloud-settings">
            <div class="setting-item">
              <div class="setting-info">
                <h3>同步频率</h3>
                <p>自动同步剪贴板历史的频率</p>
              </div>
              <div class="setting-control">
                <select 
                  v-model="settings.sync_frequency" 
                  @change="updateSetting('sync_frequency', $event.target.value)" 
                  class="select-input"
                >
                  <option value="realtime">实时同步</option>
                  <option value="5min">每5分钟</option>
                  <option value="15min">每15分钟</option>
                  <option value="1hour">每小时</option>
                </select>
              </div>
            </div>

            <div class="setting-item">
              <div class="setting-info">
                <h3>同步内容类型</h3>
                <p>同步(仅文本 / 包含图片 / 包含文件)</p>
              </div>
              <div class="setting-control">
                <select 
                  v-model="settings.sync_content_type" 
                  @change="updateSetting('sync_content_type', $event.target.value)" 
                  class="select-input"
                >
                  <option value="onlytxt">仅文本</option>
                  <option value="containphoto">包含图片</option>
                  <option value="containfile">包含文件</option>
                </select>
              </div>
            </div>

            <div class="setting-item">
              <div class="setting-info">
                <h3>加密同步数据</h3>
                <p>使用端到端加密保护您的剪贴板数据</p>
              </div>
              <div class="setting-control">
                <label class="toggle-switch">
                  <input 
                    type="checkbox" 
                    :checked="settings.encrypt_cloud_data" 
                    @change="updateSetting('encrypt_cloud_data', $event.target.checked)"
                  >
                  <span class="slider"></span>
                </label>
              </div>
            </div>
            
            <div class="account-status" v-if="!userLoggedIn">
              <p>您尚未登录，请登录以启用云端同步功能</p>  
              <div class="account-buttons">
                <button class="btn btn-secondary" @click="activeNav = 'user'">前往用户信息</button>
              </div>            
            </div>
            
            <div class="account-status" v-else>
              <p>已登录为: {{ userEmail }}</p>
              <div class="account-buttons">
                <button class="btn btn-primary" @click="activeNav = 'user'">查看用户信息</button>
              </div>
            </div>
          </div>
        </div>

        <!-- 用户信息 -->
        <div v-if="activeNav === 'user'" class="panel-section">
          <h2>用户信息</h2>
          
          <div class="user-profile">
            <div class="avatar-section">
              <div class="avatar">
                <img v-if="userInfo.avatar" :src="userInfo.avatar" alt="用户头像" class="user-avatar-img">
                <span v-else>👤</span>
              </div>
              <button class="btn btn-secondary" @click="changeAvatar">更换头像</button>
            </div>
            
            <div class="user-details">
              <div class="form-group">
                <label>用户名</label>
                <div class="display-value">{{ userInfo.username || '未登录' }}</div>
              </div>
              
              <div class="form-group">
                <label>电子邮箱</label>
                <div class="display-value">{{ userInfo.email || '无邮箱信息' }}</div>
              </div>
              
              <div class="form-group">
                <label>个人简介</label>
                <textarea 
                  :value="userInfo.bio" 
                  @input="userInfo.bio = $event.target.value"
                  @blur="updateUserInfo()"
                  class="textarea-input" 
                  rows="3"
                ></textarea>
              </div>
            </div>
          </div>
          
          <div class="account-actions">
            <h3>账户操作</h3>
            <div class="action-buttons">
              <template v-if="userLoggedIn">
                <button class="btn btn-secondary" @click.prevent="logout">退出登录</button>
                <button class="btn btn-secondary" @click="openChangePasswordDialog" :disabled="!userLoggedIn">修改密码</button>
                <button class="btn btn-danger" @click="deleteAccount" :disabled="loading">
                  <span v-if="loading">处理中...</span>
                  <span v-else>删除账户</span>
                </button>
              </template>
              
              <template v-else>
                <button class="btn btn-primary" @click="openRegisterDialog">注册账户</button>
                <button class="btn btn-secondary" @click="openLoginDialog">登录</button>
              </template>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 提示信息 -->
    <div v-if="showToast" class="toast">
      {{ toastMessage }}
    </div>

    <!-- 注册对话框 -->
    <div v-if="showRegisterDialog" class="modal-overlay">
      <div class="modal-content">
        <div class="modal-header">
          <h3>注册新账户</h3>
          <button @click="closeRegisterDialog" class="close-btn">&times;</button>
        </div>
        
        <div class="modal-body">
          <form @submit.prevent="handleRegister">
            <div class="form-group">
              <label for="username">用户名</label>
              <input
                id="username"
                v-model="registerData.username"
                type="text"
                required
                placeholder="请输入用户名（至少3个字符）"
                class="form-input"
                :class="{ 'error': registerErrors.username }"
              />
              <div v-if="registerErrors.username" class="error-message">{{ registerErrors.username }}</div>
            </div>
            
            <div class="form-group">
              <label for="email">邮箱</label>
              <input
                id="email"
                v-model="registerData.email"
                type="email"
                required
                placeholder="请输入邮箱"
                class="form-input"
                :class="{ 'error': registerErrors.email }"
              />
              <div v-if="registerErrors.email" class="error-message">{{ registerErrors.email }}</div>
            </div>
            
            <div class="form-group">
              <label for="password">密码</label>
              <input
                id="password"
                v-model="registerData.password"
                type="password"
                required
                placeholder="请输入密码（至少9位）"
                class="form-input"
                :class="{ 'error': registerErrors.password }"
              />
              <div v-if="registerErrors.password" class="error-message">{{ registerErrors.password }}</div>
            </div>
            
            <div class="form-group">
              <label for="password2">确认密码</label>
              <input
                id="password2"
                v-model="registerData.password2"
                type="password"
                required
                placeholder="请再次输入密码"
                class="form-input"
                :class="{ 'error': registerErrors.password2 }"
              />
              <div v-if="registerErrors.password2" class="error-message">{{ registerErrors.password2 }}</div>
            </div>
            
            <div class="form-actions">
              <button type="button" @click="closeRegisterDialog" class="btn btn-secondary">
                取消
              </button>
              <button type="submit" :disabled="registerLoading" class="btn btn-primary">
                <span v-if="registerLoading">注册中...</span>
                <span v-else>注册</span>
              </button>
            </div>
          </form>
        </div>
      </div>
    </div>

    <!-- 登录对话框 -->
    <div v-if="showLoginDialog" class="modal-overlay">
      <div class="modal-content">
        <div class="modal-header">
          <h3>登录账户</h3>
          <button @click="closeLoginDialog" class="close-btn">&times;</button>
        </div>
        
        <div class="modal-body">
          <form @submit.prevent="handleLogin">
            <div class="form-group">
              <label for="login-username">用户名</label>
              <input
                id="login-username"
                v-model="loginData.username"
                type="text"
                required
                placeholder="请输入用户名"
                class="form-input"
              />
            </div>
            
            <div class="form-group">
              <label for="login-password">密码</label>
              <input
                id="login-password"
                v-model="loginData.password"
                type="password"
                required
                placeholder="请输入密码"
                class="form-input"
              />
            </div>
            
            <div class="form-actions">
              <button type="button" @click="closeLoginDialog" class="btn btn-secondary">
                取消
              </button>
              <button type="submit" :disabled="loginLoading" class="btn btn-primary">
                <span v-if="loginLoading">登录中...</span>
                <span v-else>登录</span>
              </button>
            </div>
            
            <div class="form-footer">
              <p>还没有账户？ <a href="#" @click.prevent="showLoginDialog = false; openRegisterDialog()">立即注册</a></p>
            </div>
          </form>
        </div>
      </div>
    </div>

    <!-- 修改密码对话框 -->
    <div v-if="showChangePasswordDialog" class="modal-overlay">
      <div class="modal-content">
        <div class="modal-header">
          <h3>修改密码</h3>
          <button @click="closeChangePasswordDialog" class="close-btn">&times;</button>
        </div>

        <div class="modal-body">
          <form @submit.prevent="handleChangePassword">
            <div class="form-group">
              <label for="old-password">旧密码</label>
              <input
                id="old-password"
                v-model="changePasswordData.old_password"
                type="password"
                required
                placeholder="请输入旧密码"
                class="form-input"
                :class="{ 'error': changePasswordErrors.old_password }"
              />
              <div v-if="changePasswordErrors.old_password" class="error-message">{{ changePasswordErrors.old_password }}</div>
            </div>
            
            <div class="form-group">
              <label for="new-password">新密码</label>
              <input
                id="new-password"
                v-model="changePasswordData.new_password"
                type="password"
                required
                placeholder="请输入新密码（至少6位）"
                class="form-input"
                :class="{ 'error': changePasswordErrors.new_password }"
              />
              <div v-if="changePasswordErrors.new_password" class="error-message">{{ changePasswordErrors.new_password }}</div>
            </div>
            
            <div class="form-group">
              <label for="new-password2">确认新密码</label>
              <input
                id="new-password2"
                v-model="changePasswordData.new_password2"
                type="password"
                required
                placeholder="请再次输入新密码"
                class="form-input"
                :class="{ 'error': changePasswordErrors.new_password2 }"
              />
              <div v-if="changePasswordErrors.new_password2" class="error-message">{{ changePasswordErrors.new_password2 }}</div>
            </div>
            
            <div class="form-actions">
              <button type="button" @click="closeChangePasswordDialog" class="btn btn-secondary">
                取消
              </button>
              <button type="submit" :disabled="changePasswordLoading" class="btn btn-primary">
                <span v-if="changePasswordLoading">修改中...</span>
                <span v-else>确定修改</span>
              </button>
            </div>
          </form>
        </div>
      </div>
    </div>
    
  </div>
</template>

<script setup>
import { usePreferences } from '../composables/Preferences'

const {
  // 状态
  activeNav,
  showToast,
  toastMessage,
  recordingShortcut,
  newIgnoredApp,
  userLoggedIn,
  userEmail,
  autostart,
  loading,
  errorMsg,
  successMsg,
  currentShortcut,
  shortcutManager,
  recordingShortcutType,
  lastSyncTime,
  lastSyncStatus,
  isSyncing,
  userInfo,
  navItems,
  settings,
  shortcutDisplayNames,
  shortcutKeys,

  // 注册登录相关状态
  showRegisterDialog,
  showLoginDialog,
  registerData,
  loginData,
  registerErrors,
  registerLoading,
  loginLoading,

  // 修改密码相关状态
  showChangePasswordDialog,
  changePasswordData,
  changePasswordErrors,
  changePasswordLoading,

  // 基础方法
  setActiveNav,
  goBack,
  login,
  logout,
  resetUserInfo,
  showMessage,

  // 注册登录方法
  handleRegister,
  handleLogin,
  openRegisterDialog,
  openLoginDialog,
  closeRegisterDialog,
  closeLoginDialog,
  updateUserInfo,

  // 修改密码方法
  handleChangePassword,
  openChangePasswordDialog,
  closeChangePasswordDialog,
  
  // 快捷键方法
  startRecording,
  cancelRecording,
  setShortcut,

  // 设置方法
  updateSetting,
  toggleOCRLanguage,
  changeStoragePath,

  // 数据管理方法
  clearAiHistory,
  exportData,
  importData,
  createBackup,

  // 隐私管理方法
  showPrivate,
  
  // 云端同步方法
  formatTime,
  manualSync,
  syncNow,
  checkSyncStatus,
  handleCloudPush,

  // 用户管理方法
  changeAvatar,
  changePassword,
  deleteAccount,

  // 辅助方法
  getAIServiceName,
  getBackupFrequencyName
} = usePreferences()
</script>

<style scoped>
* {
  box-sizing: border-box;
}

.settings-container {
  min-height: 100vh;
  background: white;
  overflow-x: hidden;
  max-width: 100%;
  width: 100vw;
  position: fixed;
}

/* 设置头部样式 */
.settings-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 6px 8px;
  border-bottom: 1px solid #e1e8ed;
  background: white;
  max-width: 100%;
}

.settings-header h1 {
  font-size: 15px;
  font-weight: 600;
  color: #2c3e50;
}

.back-btn {
  padding: 6px 8px;
  border: 1px solid #e1e8ed;
  border-radius: 6px;
  background: white;
  color: #3498db;
  font-size: 14px;
  cursor: pointer;
  transition: all 0.2s;
}

.back-btn:hover {
  background: #f8f9fa;
  border-color: #3498db;
}

/* 设置内容区域 */
.settings-content {
  display: flex;
  height: calc(100vh - 40px);
  max-width: 100%;
}

/* 左侧导航栏 */
.settings-nav {
  width: 200px;
  border-right: 1px solid #e1e8ed;
  background: #f8f9fa;
  overflow-y: auto;
  padding: 6px 8px;
}

.nav-list {
  list-style: none;
  padding: 0;
  margin: 0;
}

.nav-item {
  display: flex;
  align-items: center;
  padding: 12px 16px;
  cursor: pointer;
  transition: all 0.1s;
  border: none;
  border-radius: 8px;
  gap: 8px;
}

.nav-item:hover {
  background: #f1f3f5;
}

.nav-item.active {
  background: #e4edfd;
  color: #416afe;
}

.nav-icon {
  width: 1.2rem;
  height: 1.2rem;
  position: relative;
  top: 1px; 
}

.nav-text {
  font-size: 14px;
  font-weight: 500;
}

/* 右侧设置面板 */
.settings-panel {
  flex: 1;
  padding: 24px;
  overflow-y: auto;
  background: white;
}

.panel-section h2 {
  margin-bottom: 24px;
  font-size: 20px;
  font-weight: 600;
  color: #2c3e50;
  border-bottom: 1px solid #e1e8ed;
  padding-bottom: 12px;
}

.setting-item {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  padding: 16px 0;
  border-bottom: 1px solid #f0f0f0;
}

.setting-info h3 {
  margin: 0 0 4px 0;
  font-size: 15px;
  font-weight: 500;
  color: #2c3e50;
}

.setting-info p {
  margin: 0;
  font-size: 13px;
  color: #7f8c8d;
}

.setting-control {
  display: flex;
  align-items: center;
  min-width: 160px;
  justify-content: flex-end;
}

/* 切换开关样式 */
.toggle-switch {
  position: relative;
  display: inline-block;
  width: 44px;
  height: 24px;
}

.toggle-switch input {
  opacity: 0;
  width: 0;
  height: 0;
}

.slider {
  position: absolute;
  cursor: pointer;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: #ccc;
  transition: .4s;
  border-radius: 24px;
}

.slider:before {
  position: absolute;
  content: "";
  height: 18px;
  width: 18px;
  left: 3px;
  bottom: 3px;
  background-color: white;
  transition: .4s;
  border-radius: 50%;
}

input:checked + .slider {
  background-color: #3498db;
}

input:checked + .slider:before {
  transform: translateX(20px);
}

/* 输入框样式 */
.select-input, .number-input, .text-input, .textarea-input {
  padding: 8px 12px;
  border: 1px solid #e1e8ed;
  border-radius: 6px;
  font-size: 14px;
  outline: none;
  transition: border-color 0.2s;
}

.select-input:focus, .number-input:focus, .text-input:focus, .textarea-input:focus {
  border-color: #3498db;
}

.number-input {
  width: 80px;
}

.text-input, .textarea-input {
  width: 100%;
}

.unit {
  margin-left: 8px;
  font-size: 14px;
  color: #7f8c8d;
}

/* 标签输入样式 */
.tag-input-container {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  align-items: center;
  min-width: 200px;
}

.tag {
  display: flex;
  align-items: center;
  background: #edf3fe;
  color: #3498db;
  padding: 4px 8px;
  border-radius: 4px;
  font-size: 12px;
}

.tag-remove {
  margin-left: 4px;
  cursor: pointer;
  font-weight: bold;
}

.tag-input {
  flex: 1;
  min-width: 120px;
  padding: 4px 8px;
  border: 1px solid #e1e8ed;
  border-radius: 4px;
  font-size: 12px;
}

/* 快捷键输入样式 */
.shortcut-input {
  padding: 8px 12px;
  border: 1px solid #e1e8ed;
  border-radius: 6px;
  background: white;
  cursor: pointer;
  text-align: center;
  min-width: 120px;
  transition: all 0.2s;
  user-select: none;
}

.shortcut-input:hover {
  border-color: #3498db;
  background: #f8f9fa;
}

.shortcut-status-messages {
    margin-top: 24px;
}

.shortcut-input.recording-active {
  border-color: #e67e22; /* Orange color for active recording */
  background: #fdf3e9; /* Light orange background */
  box-shadow: 0 0 5px rgba(230, 126, 34, 0.5);
  animation: pulse-border 1s infinite alternate;
}

.hint {
  margin-top: 24px;
  padding: 12px;
  background: #f8f9fa;
  border-radius: 6px;
  font-size: 13px;
  color: #7f8c8d;
}

/* AI设置样式 */
.ai-settings {
  margin-top: 16px;
  padding-top: 16px;
  border-top: 1px solid #f0f0f0;
}

.checkbox-group {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.checkbox-item {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 14px;
  cursor: pointer;
}

.checkbox-item input[type="checkbox"] {
  margin: 0;
}

/* 备份设置样式 */
.path-input-container {
  width: 100%;
  max-width: 400px;
}

.path-input-group {
  display: flex;
  width: 100%;
  gap: 8px;
}

.path-input {
  flex: 1;
  min-width: 200px;
  background: #f8f9fa;
  cursor: pointer;
  transition: background-color 0.2s;
  border: 1px solid #e1e8ed;
}

.path-input:hover {
  background: #e9ecef;
  border-color: #3498db;
}

.path-btn {
  flex-shrink: 0;
  white-space: nowrap;
  min-width: 100px;
}

.path-hint {
  margin-top: 4px;
  color: #6c757d;
  font-size: 12px;
}

/* 调整设置项布局 */
.setting-item {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  padding: 16px 0;
  border-bottom: 1px solid #f0f0f0;
  gap: 20px; /* 添加间距 */
}

.setting-info {
  flex: 1;
  min-width: 200px;
}

.setting-control {
  flex: 1;
  min-width: 300px;
  display: flex;
  align-items: flex-start;
  justify-content: flex-end;
}

/* 确保备份操作项也正确显示 */
.backup-actions .action-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px;
  border: 1px solid #e1e8ed;
  border-radius: 8px;
  background: #f8f9fa;
  gap: 20px;
}

.action-info {
  flex: 1;
}

.action-item .btn {
  flex-shrink: 0;
}

/* 云端设置样式 */
.cloud-settings {
  margin-top: 16px;
  padding-top: 16px;
  border-top: 1px solid #f0f0f0;
}

.account-status {
  margin-top: 24px;
  padding: 16px;
  background: #f8f9fa;
  border-radius: 8px;
  text-align: center;
}

.account-status p {
  margin-bottom: 12px;
  font-size: 14px;
  color: #2c3e50;
}

.sync-status {
  background: #f8f9fa;
  border: 1px solid #e9ecef;
  border-radius: 8px;
  padding: 16px;
  margin-bottom: 20px;
}

.status-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.status-label {
  font-weight: 500;
  color: #6c757d;
  font-size: 14px;
}

.status-value {
  font-weight: 500;
  font-size: 14px;
}

.status-value.success {
  color: #28a745;
}

.status-value.error {
  color: #dc3545;
}

.status-actions {
  display: flex;
  justify-content: flex-end;
  margin-top: 12px;
}

.btn-small {
  padding: 6px 12px;
  font-size: 14px;
}

.btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

/* 用户信息样式 */
.user-profile {
  display: flex;
  gap: 24px;
  margin-bottom: 32px;
}

.avatar-section {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
}

.avatar {
  width: 80px;
  height: 80px;
  border-radius: 50%;
  background: #edf3fe;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 32px;
  overflow: hidden; /* 隐藏超出圆形区域的部分 */
  position: relative; /* 为绝对定位的图片做准备 */
  border: 2px solid #e1e8ed;/* 添加边框增强圆形效果 */
}

.user-avatar-img {
  width: 100%;
  height: 100%;
  object-fit: cover; /* 确保图片覆盖整个容器并保持比例 */
  object-position: center center; /* 确保图片居中显示 */
  display: block;
}

.user-details {
  flex: 1;
}

.form-group {
  margin-bottom: 16px;
}

.form-group label {
  display: block;
  margin-bottom: 6px;
  font-size: 14px;
  font-weight: 500;
  color: #2c3e50;
}

.form-actions {
  display: flex;
  gap: 12px;
  margin-top: 24px;
}

.account-actions {
  padding-top: 24px;
  border-top: 1px solid #f0f0f0;
}

.account-actions h3 {
  margin-bottom: 16px;
  font-size: 16px;
  font-weight: 500;
  color: #2c3e50;
}

.action-buttons {
  display: flex;
  gap: 12px;
}

/* 按钮样式 */
.btn {
  padding: 8px 16px;
  border: none;
  border-radius: 6px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
}

.btn-primary {
  background: #3498db;
  color: white;
}

.btn-primary:hover {
  background: #2980b9;
}

.btn-secondary {
  background: #ecf0f1;
  color: #2c3e50;
  border: 1px solid #bdc3c7;
}

.btn-secondary:hover {
  background: #d5dbdb;
}

.btn-danger {
  background: #e74c3c;
  color: white;
}

.btn-danger:hover {
  background: #c0392b;
}

/* 未登录用户界面 */
.unlogged-user {
  padding: 40px 20px;
  text-align: center;
  background: #f8f9fa;
  border-radius: 8px;
  border: 1px solid #e1e8ed;
}

.unlogged-message h3 {
  margin-bottom: 10px;
  color: #2c3e50;
  font-size: 18px;
}

.unlogged-message p {
  margin-bottom: 20px;
  color: #7f8c8d;
}

.unlogged-buttons {
  display: flex;
  gap: 10px;
  justify-content: center;
}

.display-value {
  padding: 8px 12px;
  border: 1px solid #e1e8ed;
  border-radius: 6px;
  font-size: 14px;
  color: #2c3e50;
  background: #f8f9fa; /* Light background to make it look like a static display field */
  word-break: break-all;
}

/* 账户按钮组 */
.account-buttons {
  display: flex;
  gap: 10px;
  margin-top: 15px;
}

/* 模态框样式 */
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: rgba(0, 0, 0, 0.5);
  display: flex;
  justify-content: center;
  align-items: center;
  z-index: 2000;
}

.modal-content {
  background: white;
  border-radius: 8px;
  width: 90%;
  max-width: 400px;
  max-height: 80vh;
  overflow-y: auto;
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px;
  border-bottom: 1px solid #eee;
}

.modal-header h3 {
  margin: 0;
  font-size: 18px;
  color: #2c3e50;
}

.close-btn {
  background: none;
  border: none;
  font-size: 24px;
  cursor: pointer;
  color: #666;
}

.modal-body {
  padding: 20px;
}

/* 表单样式 */
.form-group {
  margin-bottom: 20px;
}

.form-group label {
  display: block;
  margin-bottom: 5px;
  font-weight: 500;
  color: #2c3e50;
}

.form-input {
  width: 100%;
  padding: 10px;
  border: 1px solid #ddd;
  border-radius: 4px;
  font-size: 14px;
}

.form-input:focus {
  outline: none;
  border-color: #3498db;
  box-shadow: 0 0 0 2px rgba(52, 152, 219, 0.25);
}

.form-input.error {
  border-color: #e74c3c;
}

.error-message {
  color: #e74c3c;
  font-size: 12px;
  margin-top: 5px;
}

.form-actions {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
  margin-top: 30px;
}

.form-footer {
  margin-top: 20px;
  text-align: center;
  font-size: 14px;
  color: #7f8c8d;
}

.form-footer a {
  color: #3498db;
  text-decoration: none;
}

.form-footer a:hover {
  text-decoration: underline;
}

/* 按钮样式更新 */
.btn {
  padding: 8px 16px;
  border: none;
  border-radius: 6px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
}

.btn-primary {
  background: #3498db;
  color: white;
}

.btn-primary:hover:not(:disabled) {
  background: #2980b9;
}

.btn-primary:disabled {
  background: #a0c9e5;
  cursor: not-allowed;
}

.btn-secondary {
  background: #ecf0f1;
  color: #2c3e50;
  border: 1px solid #bdc3c7;
}

.btn-secondary:hover {
  background: #d5dbdb;
}

.btn-danger {
  background: #e74c3c;
  color: white;
}

.btn-danger:hover {
  background: #c0392b;
}

.btn-small {
  padding: 6px 12px;
  font-size: 14px;
}

/* 提示信息样式 */
.toast {
  position: fixed;
  bottom: 24px;
  left: 50%;
  transform: translateX(-50%);
  background: rgba(0, 0, 0, 0.8);
  color: white;
  padding: 12px 24px;
  border-radius: 8px;
  font-size: 14px;
  z-index: 10000;
  animation: slideUp 0.3s ease;
}

.tip-text {
  font-size: 0.9em; 
  color: #888; 
  margin-left: 0px; 
}

@keyframes slideUp {
  from {
    opacity: 0;
    transform: translateX(-50%) translateY(20px);
  }
  to {
    opacity: 1;
    transform: translateX(-50%) translateY(0);
  }
}

@keyframes pulse-border {
  from {
    border-color: #e67e22;
  }
  to {
    border-color: #f1c40f;
  }
}

/* 响应式设计 */
@media (max-width: 768px) {
  .settings-content {
    flex-direction: column;
    height: auto;
  }
  
  .settings-nav {
    width: 100%;
    border-right: none;
    border-bottom: 1px solid #e1e8ed;
  }
  
  .nav-list {
    display: flex;
    overflow-x: auto;
  }
  
  .nav-item {
    flex-shrink: 0;
    border-left: none;
    border-bottom: 3px solid transparent;
  }
  
  .nav-item.active {
    border-left-color: transparent;
    border-bottom-color: #3498db;
  }
  
  .setting-item {
    flex-direction: column;
    align-items: flex-start;
  }
  
  .setting-control {
    margin-top: 12px;
    width: 100%;
    justify-content: flex-start;
  }
  
  .user-profile {
    flex-direction: column;
  }
  
  .avatar-section {
    align-self: center;
  }
}

/* 滑块输入样式 */
.slider-container {
  display: flex;
  align-items: center;
  gap: 12px;
  min-width: 200px;
}

.slider-input {
  flex: 1;
  height: 6px;
  border-radius: 3px;
  background: #e1e8ed;
  outline: none;
  -webkit-appearance: none;
}

.slider-input::-webkit-slider-thumb {
  -webkit-appearance: none;
  width: 18px;
  height: 18px;
  border-radius: 50%;
  background: #3498db;
  cursor: pointer;
}

.slider-input::-moz-range-thumb {
  width: 18px;
  height: 18px;
  border-radius: 50%;
  background: #3498db;
  cursor: pointer;
  border: none;
}

.slider-value {
  min-width: 40px;
  text-align: center;
  font-size: 14px;
  color: #2c3e50;
}

</style>