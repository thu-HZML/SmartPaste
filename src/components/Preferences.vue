<template>
  <div class="settings-container">
    <!-- 设置头部 -->
    <header class="settings-header">
      <h1>设置</h1>
      <button class="back-btn" @click="goBack">← 返回</button>
    </header>

    <!-- 设置内容区域 -->
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
              <h3>启动时最小化到托盘</h3>
              <p>启动时不弹出窗口，挂载在后台</p>
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
              <h3>显示系统托盘图标</h3>
              <p>在系统托盘显示应用图标，方便快速访问</p>
            </div>
            <div class="setting-control">
              <label class="toggle-switch">
                <input 
                  type="checkbox" 
                  :checked="settings.minimize_to_tray" 
                  @change="updateSetting('minimize_to_tray', $event.target.checked)"
                >
                <span class="slider"></span>
              </label>
            </div>
          </div>
          
          <div class="setting-item">
            <div class="setting-info">
              <h3>自动保存剪贴板历史</h3>
              <p>自动保存剪贴板内容到历史记录</p>
            </div>
            <div class="setting-control">
              <label class="toggle-switch">
                <input 
                  type="checkbox" 
                  :checked="settings.auto_save" 
                  @change="updateSetting('auto_save', $event.target.checked)"
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
              <h3>自动分类</h3>
              <p>自动分类开关</p>
            </div>
            <div class="setting-control">
              <label class="toggle-switch">
                <input 
                  type="checkbox" 
                  :checked="settings.auto_classify" 
                  @change="updateSetting('auto_classify', $event.target.checked)"
                >
                <span class="slider"></span>
              </label>
            </div>
          </div>

          <div class="setting-item">
            <div class="setting-info">
              <h3>OCR自动识别</h3>
              <p>OCR自动识别开关</p>
            </div>
            <div class="setting-control">
              <label class="toggle-switch">
                <input 
                  type="checkbox" 
                  :checked="settings.ocr_auto_recognition" 
                  @change="updateSetting('ocr_auto_recognition', $event.target.checked)"
                >
                <span class="slider"></span>
              </label>
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
                  v-model="settings.ai_service" 
                  @change="updateSetting('ai_service', $event.target.value)" 
                  class="select-input"
                >
                  <option value="openai">OpenAI</option>
                  <option value="claude">Claude</option>
                  <option value="gemini">Gemini</option>
                  <option value="deepseek">DeepSeek</option>
                  <option value="custom">自定义</option>
                </select>
              </div>
            </div>

            <div class="setting-item">
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

            <div class="setting-item">
              <div class="setting-info">
                <h3>清空AI对话历史</h3>
                <p>清除所有AI对话记录</p>
              </div>
              <div class="setting-control">
                <button class="btn btn-secondary" @click="clearAiHistory">清空历史</button>
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
                  > 密码
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
              <button class="btn btn-secondary">查看隐私记录</button>
            </div>
          </div>

          <div class="setting-item">
            <div class="setting-info">
              <h3>自动清理隐私记录</h3>
              <p>自动删除超过指定天数的隐私记录</p>
            </div>
            <div class="setting-control">
              <select 
                v-model="settings.privacy_retention_days" 
                @change="updateSetting('privacy_retention_days', Number($event.target.value))" 
                class="select-input"
              >
                <option value="1">1天</option>
                <option value="7">7天</option>
                <option value="30">30天</option>
                <option value="0">手动删除</option>
              </select>
            </div>
          </div>

          <div class="setting-item">
            <div class="setting-info">
              <h3>删除所有隐私记录</h3>
              <p>永久删除所有标记为隐私的记录</p>
            </div>
            <div class="setting-control">
              <button class="btn btn-danger">删除所有隐私记录</button>
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

          <div class="setting-item">
            <div class="setting-info">
              <h3>自动备份</h3>
              <p>定期自动备份数据</p>
            </div>
            <div class="setting-control">
              <label class="toggle-switch">
                <input 
                  type="checkbox" 
                  :checked="settings.auto_backup" 
                  @change="updateSetting('auto_backup', $event.target.checked)"
                >
                <span class="slider"></span>
              </label>
            </div>
          </div>

          <div v-if="settings.auto_backup" class="setting-item">
            <div class="setting-info">
              <h3>备份频率</h3>
              <p>自动备份的频率</p>
            </div>
            <div class="setting-control">
              <select 
                v-model="settings.backup_frequency" 
                @change="updateSetting('backup_frequency', $event.target.value)" 
                class="select-input"
              >
                <option value="daily">每天</option>
                <option value="weekly">每周</option>
                <option value="monthly">每月</option>
              </select>
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

              <div class="action-item">
                <div class="action-info">
                  <h4>立即备份</h4>
                  <p>立即创建数据备份</p>
                </div>
                <button class="btn btn-secondary" @click="createBackup">立即备份</button>
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
              <button class="btn btn-small" @click="manualSync" :disabled="isSyncing">
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

            <div class="setting-item">
              <div class="setting-info">
                <h3>仅WiFi下同步</h3>
                <p>仅WiFi下同步</p>
              </div>
              <div class="setting-control">
                <label class="toggle-switch">
                  <input 
                    type="checkbox" 
                    :checked="settings.sync_only_wifi" 
                    @change="updateSetting('sync_only_wifi', $event.target.checked)"
                  >
                  <span class="slider"></span>
                </label>
              </div>
            </div>
            
            <div class="account-status" v-if="!userLoggedIn">
              <p>您尚未登录，请登录以启用云端同步功能</p>
              <button class="btn btn-primary" @click="login">登录账户</button>
            </div>
            
            <div class="account-status" v-else>
              <p>已登录为: {{ userEmail }}</p>
              <button class="btn btn-secondary" @click="logout">退出登录</button>
            </div>
          </div>
        </div>

        <!-- 用户信息 -->
        <div v-if="activeNav === 'user'" class="panel-section">
          <h2>用户信息</h2>
          
          <div class="user-profile">
            <div class="avatar-section">
              <div class="avatar">👤</div>
              <button class="btn btn-secondary">更换头像</button>
            </div>
            
            <div class="user-details">
              <div class="form-group">
                <label>用户名</label>
                <input 
                  type="text" 
                  :value="settings.username" 
                  @input="settings.username = $event.target.value"
                  @blur="updateSetting('username', $event.target.value)"
                  class="text-input"
                >
              </div>
              
              <div class="form-group">
                <label>电子邮箱</label>
                <input 
                  type="text" 
                  :value="settings.email" 
                  @input="settings.email = $event.target.value"
                  @blur="updateSetting('email', $event.target.value)"
                  class="text-input"
                >
              </div>
              
              <div class="form-group">
                <label>个人简介</label>
                <textarea 
                  :value="settings.bio" 
                  @input="settings.bio = $event.target.value"
                  @blur="updateSetting('bio', $event.target.value)"
                  class="textarea-input" 
                  rows="3"
                ></textarea>
              </div>
            </div>
          </div>
          
          <div class="account-actions">
            <h3>账户操作</h3>
            <div class="action-buttons">
              <button class="btn btn-secondary" @click="changePassword">修改密码</button>
              <button class="btn btn-danger" @click="deleteAccount">删除账户</button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 提示信息 -->
    <div v-if="showToast" class="toast">
      {{ toastMessage }}
    </div>
  </div>
</template>

<script setup>
import { ref, reactive, onMounted,computed } from 'vue'
import { useRouter } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import { getCurrentWindow, LogicalSize } from '@tauri-apps/api/window';
import { open } from '@tauri-apps/plugin-dialog';
import { useSettingsStore } from '../stores/settings'
import { 
  Cog6ToothIcon,
  TvIcon,
  CloudIcon,
  ClipboardIcon,
  UserIcon,
  SparklesIcon,     // 新增
  ShieldCheckIcon,   // 新增
  ArchiveBoxIcon
 } from '@heroicons/vue/24/outline'

const router = useRouter()
const currentWindow = getCurrentWindow();

// 响应式数据
const activeNav = ref('general')
const showToast = ref(false)
const toastMessage = ref('')
const recordingShortcut = ref('')
const newIgnoredApp = ref('')
const userLoggedIn = ref(true)
const userEmail = ref('user@example.com')

const autostart = ref(false)
const loading = ref(false)

// 添加快捷键设置所需的变量
const errorMsg = ref('')
const successMsg = ref('')
const currentShortcut = ref('')
let timer = null
const shortcutManager = reactive({
  currentType: '',
  isRecording: false,
  currentKeys: new Set()
})
const recordingShortcutType = ref('') // 当前正在录制的快捷键类型

// 导航项
const navItems = ref([
  { id: 'general', name: '通用设置', icon: Cog6ToothIcon },
  { id: 'shortcuts', name: '快捷键设置', icon: TvIcon },
  { id: 'clipboard', name: '剪贴板参数设置', icon: ClipboardIcon },
  { id: 'ocr', name: 'OCR设置', icon: ClipboardIcon },
  { id: 'ai', name: 'AI Agent 设置', icon: ClipboardIcon },
  { id: 'security', name: '安全与隐私', icon: ClipboardIcon }, 
  { id: 'backup', name: '数据备份', icon: ClipboardIcon },
    { id: 'cloud', name: '云端入口', icon: CloudIcon },
  { id: 'user', name: '用户信息', icon: UserIcon }
  
])

// 设置数据
const settings = useSettingsStore().settings

// 快捷键显示名称映射
const shortcutDisplayNames = {
  global_shortcut: '显示/隐藏主窗口',
  global_shortcut_2: '显示/隐藏剪贴板', 
  global_shortcut_3: '显示/隐藏AI助手',
  global_shortcut_4: '快速粘贴',
  global_shortcut_5: '清空剪贴板历史'
}

// 同步状态相关数据
const lastSyncTime = ref(null) // 上次同步时间戳
const lastSyncStatus = ref('') // 'success', 'error', ''
const isSyncing = ref(false) // 是否正在同步

// 用户信息
const userInfo = reactive({
  username: '当前用户',
  email: 'user@example.com',
  bio: '剪贴板管理爱好者'
})

// 方法定义
const setActiveNav = (navId) => {
  activeNav.value = navId
}

const goBack = () => {
  router.back()
}

const login = () => {
  // 模拟登录
  userLoggedIn.value = true
  showMessage('登录成功')
}

const logout = () => {
  userLoggedIn.value = false
  showMessage('已退出登录')
}

const resetUserInfo = () => {
  Object.assign(userInfo, {
    username: '当前用户',
    email: 'user@example.com',
    bio: '剪贴板管理爱好者'
  })
  showMessage('用户信息已重置')
}


const showMessage = (message) => {
  toastMessage.value = message
  showToast.value = true
  setTimeout(() => {
    showToast.value = false
  }, 2000)
}

// 生命周期
onMounted(async () => {
  // 加载保存的设置
  /*
  const savedSettings = localStorage.getItem('clipboardSettings')
  if (savedSettings) {
    Object.assign(settings, JSON.parse(savedSettings))
  }
  const savedTime = localStorage.getItem('lastSyncTime');
  if (savedTime) {
    lastSyncTime.value = parseInt(savedTime);
  }*/

  // await checkAutostartStatus()

  // 初始化窗口大小
  try {
    await currentWindow.setSize(new LogicalSize(800, 580));
  } catch (error) {
    console.error('设置窗口大小失败:', error)
  }
})

// 通用设置相关函数
// 启动时自动运行
// 检查自启状态
/*
const checkAutostartStatus = async () => {
  try {
    const isEnabled = await invoke('is_autostart_enabled')
    settings.autoStart = isEnabled
    console.log('当前自启状态:', isEnabled)
  } catch (error) {
    console.error('检查自启状态失败:', error)
    showMessage('检查自启状态失败')
  }
}

// 切换自启状态 - 唯一的函数
const toggleAutoStart = async () => {
  loading.value = true
  try {
    await invoke('set_autostart', { enable: settings.autoStart })
    const message = settings.autoStart ? '已开启开机自启' : '已关闭开机自启'
    console.log(message)
    showMessage(message)
  } catch (error) {
    console.error('设置自启失败:', error)
    showMessage(`设置失败: ${error}`)
    // 出错时恢复原状态
    settings.autoStart = !settings.autoStart
  } finally {
    loading.value = false
  }
}
// 显示系统托盘图标
const toggleTrayIcon = async () => {
  try {
    await invoke('set_tray_icon_visibility', { visible: settings.showTrayIcon })
    showMessage(settings.showTrayIcon ? '已显示托盘图标' : '已隐藏托盘图标')
  } catch (error) {
    console.error('设置托盘图标失败:', error)
    settings.showTrayIcon = !settings.showTrayIcon
    showMessage(`设置失败: ${error}`)
  }
}

//启动时最小化到托盘
const toggleMinimizeToTray = async () => {
  try {
    await invoke('set_minimize_to_tray', { enabled: settings.showTrayIcon })
    showMessage(settings.showTrayIcon ? '已启用启动时最小化到托盘' : '已禁用启动时最小化到托盘')
  } catch (error) {
    console.error('设置最小化到托盘失败:', error)
    settings.showTrayIcon = !settings.showTrayIcon
    showMessage(`设置失败: ${error}`)
  }
}

// 自动保存剪贴板历史
const toggleAutoSave = async () => {
  try {
    await invoke('set_auto_save', { enabled: settings.autoSave })
    showMessage(settings.autoSave ? '已启用自动保存' : '已禁用自动保存')
  } catch (error) {
    console.error('设置自动保存失败:', error)
    settings.autoSave = !settings.autoSave
    showMessage(`设置失败: ${error}`)
  }
}

// 历史记录保留时间
const updateRetentionDays = async () => {
  try {
    await invoke('set_retention_days', { days: parseInt(settings.retentionDays) })
    showMessage(`历史记录保留时间已设置为 ${settings.retentionDays} 天`)
  } catch (error) {
    console.error('设置保留时间失败:', error)
    showMessage(`设置失败: ${error}`)
  }
}*/

// 快捷键设置相关函数
// 开始录制快捷键
const startRecording = (shortcutType) => {
  shortcutManager.currentType = shortcutType
  shortcutManager.isRecording = true
  shortcutManager.currentKeys.clear()
  
  showMessage(`请按下 ${shortcutDisplayNames[shortcutType]} 的快捷键...`)
  
  // 添加全局事件监听
  window.addEventListener('keydown', handleKeyDownDuringRecording)
  window.addEventListener('keyup', handleKeyUpDuringRecording)
}

// 处理录制期间的按键
const handleKeyDownDuringRecording = (event) => {
  if (!shortcutManager.isRecording) return
  
  event.preventDefault()
  event.stopPropagation()
  
  const key = getKeyName(event)
  if (key) {
    shortcutManager.currentKeys.add(key)
  }
  
  // 按ESC取消
  if (event.key === 'Escape') {
    cancelRecording()
    return
  }
  
  // 检测是否有普通键被按下
  const hasRegularKey = Array.from(shortcutManager.currentKeys).some(key => 
    !['Ctrl', 'Alt', 'Shift', 'Meta'].includes(key)
  )
  
  if (hasRegularKey && shortcutManager.currentKeys.size > 0) {
    const shortcutStr = Array.from(shortcutManager.currentKeys).join('+')
    finishRecording(shortcutStr)
  }
}

// 统一的按键释放处理
const handleKeyUpDuringRecording = (event) => {
  if (!shortcutManager.isRecording) return
  
  const key = getKeyName(event)
  if (key) {
    shortcutManager.currentKeys.delete(key)
  }
}

// 获取按键名称
const getKeyName = (event) => {
  if (event.key === 'Control') return 'Ctrl'
  if (event.key === 'Alt') return 'Alt'
  if (event.key === 'Shift') return 'Shift'
  if (event.key === 'Meta') return 'Meta'
  
  // 排除修饰键
  if (event.key === 'Control' || event.key === 'Alt' || 
      event.key === 'Shift' || event.key === 'Meta') {
    return null
  }
  
  // 处理特殊按键
  if (event.key === ' ') return 'Space'
  if (event.key === 'Escape') return 'Escape'
  
  // 处理功能键
  if (event.key.startsWith('F') && event.key.length > 1) {
    const fNumber = event.key.slice(1)
    if (!isNaN(fNumber)) {
      return event.key
    }
  }
  
  // 处理字母键（转换为大写）
  if (event.key.length === 1 && event.key.match(/[a-zA-Z]/)) {
    return event.key.toUpperCase()
  }
  
  // 处理数字键
  if (event.key.match(/^[0-9]$/)) {
    return event.key
  }
  
  // 处理其他常见按键
  const specialKeys = {
    'ArrowUp': 'Up',
    'ArrowDown': 'Down', 
    'ArrowLeft': 'Left',
    'ArrowRight': 'Right',
    'Enter': 'Enter',
    'Tab': 'Tab',
    'CapsLock': 'CapsLock',
    'Backspace': 'Backspace',
    'Delete': 'Delete',
    'Insert': 'Insert',
    'Home': 'Home',
    'End': 'End',
    'PageUp': 'PageUp',
    'PageDown': 'PageDown',
    ' ': 'Space'
  }
  
  return specialKeys[event.key] || event.key
}

// 完成录制并设置快捷键
const finishRecording = async (newShortcut) => {
  shortcutManager.isRecording = false
  
  // 移除事件监听
  window.removeEventListener('keydown', handleKeyDownDuringRecording)
  window.removeEventListener('keyup', handleKeyUpDuringRecording)
  
  // 调用统一的设置函数
  await setShortcut(newShortcut, shortcutManager.currentType)
  shortcutManager.currentType = ''
}

const setShortcut = async (newShortcutStr, shortcutType) => {
  if (!shortcutType) {
    console.error('没有指定快捷键类型')
    return
  }
  
  errorMsg.value = ''
  successMsg.value = ''

  try {
    await invoke('update_shortcut', { 
      shortcutType: shortcutType,
      newShortcutStr: newShortcutStr 
    })

    // 更新界面显示
    settings.shortcuts[shortcutType] = newShortcutStr
    successMsg.value = `${shortcutDisplayNames[shortcutType]} 快捷键设置成功！`
    console.log(`✅ ${shortcutDisplayNames[shortcutType]} 快捷键已更新为: ${newShortcutStr}`)

  } catch (err) {
    errorMsg.value = `设置失败: ${err}`
    console.error('❌ 设置快捷键失败:', err)
    
    if (err.includes('Failed to unregister hotkey') || err.includes('GlobalHotkey') || err.includes('可能已被占用')) {
      errorMsg.value = '快捷键设置失败：可能与其他程序冲突，请尝试其他组合键'
    }
  }

  // 自动清除提示消息
  if (timer) clearTimeout(timer)
  timer = setTimeout(() => {
    successMsg.value = ''
    errorMsg.value = ''
  }, 3000)
}

// 取消录制（可选）
const cancelRecording = () => {
  shortcutManager.isRecording = false
  shortcutManager.currentType = ''
  window.removeEventListener('keydown', handleKeyDownDuringRecording)
  window.removeEventListener('keyup', handleKeyUpDuringRecording)
  showMessage('已取消快捷键设置')
}

/*
// 剪贴板参数设置相关函数
// 最大历史记录数量
const updateMaxHistoryItems = async () => {
  try {
    await invoke('set_max_history_items', { maxItems: settings.maxHistoryItems })
    showMessage(`最大历史记录数量已设置为 ${settings.maxHistoryItems}`)
  } catch (error) {
    console.error('设置最大历史记录数量失败:', error)
    showMessage(`设置失败: ${error}`)
  }
}

// 忽略短文本
const updateIgnoreShortText = async () => {
  try {
    await invoke('set_ignore_short_text', { minLength: settings.ignoreShortText })
    showMessage(`已设置忽略 ${settings.ignoreShortText} 字符以下的文本`)
  } catch (error) {
    console.error('设置忽略短文本失败:', error)
    showMessage(`设置失败: ${error}`)
  }
}

// 忽略大文件
const updateIgnoreBigFile = async () => {
  try {
    await invoke('set_ignore_big_file', { mincapacity: settings.ignoreBigFile })
    showMessage(`已设置忽略 ${settings.ignoreBigFile} MB以上的文件`)
  } catch (error) {
    console.error('设置忽略大文件失败:', error)
    showMessage(`设置失败: ${error}`)
  }
}*/

// 统一的设置更新函数
const updateSetting = async (key, value) => {
  const oldValue = settings[key];
  
  try {
    // 更新前端状态
    settings[key] = value;
    
    // 调用后端统一接口
    await invoke('set_config_item', { key, value });
    
    // 显示成功消息
    showMessage('设置已更新');
  } catch (error) {
    console.error(`设置 ${key} 失败:`, error);
    // 出错时回滚状态
    settings[key] = oldValue;
    showMessage(`设置失败: ${error}`);
  }
};

// 多选框辅助函数：切换OCR语言选择
const toggleOCRLanguage = async (language, isChecked) => {
  let updatedLanguages;
  
  if (isChecked) {
    // 添加语言到数组
    updatedLanguages = [...settings.ocr_languages, language];
  } else {
    // 从数组中移除语言
    updatedLanguages = settings.ocr_languages.filter(lang => lang !== language);
  }
  
  try {
    await updateSetting('ocr_languages', updatedLanguages);
    showMessage('OCR语言设置已更新');
  } catch (error) {
    console.error('更新OCR语言失败:', error);
    showMessage(`更新失败: ${error}`);
  }
};

// 修改存储路径函数
const changeStoragePath = async () => {
  try {
    // 打开目录选择对话框
    const selectedPath = await open({
      directory: true,
      multiple: false,
      title: '选择数据存储路径',
      defaultPath: settings.storage_path || undefined
    });

    // 如果用户选择了路径
    if (selectedPath) {
      // 更新前端状态
      settings.storage_path = selectedPath;
      
      // 调用后端函数更新设置
      await updateSetting('storage_path', selectedPath);
      
      showMessage('存储路径已更新');
    }
  } catch (error) {
    console.error('选择存储路径失败:', error);
    showMessage(`选择路径失败: ${error}`);
  }
};

// 删除AI对话历史
const clearAiHistory = async () => {
  if (confirm('确定要清空所有AI对话历史吗？此操作不可恢复。')) {
    try {
      // await invoke('clear_ai_history')
      showMessage('AI对话历史已清空')
    } catch (error) {
      console.error('清空AI历史失败:', error)
      showMessage(`清空失败: ${error}`)
    }
  }
}

// 导出备份函数
const exportData = async () => {
  try {
    // const exportPath = await invoke('export_user_data')
    showMessage(`数据已导出到: ${exportPath}`)
  } catch (error) {
    console.error('导出数据失败:', error)
    showMessage(`导出失败: ${error}`)
  }
}

// 导入备份函数
const importData = async () => {
  try {
    // const result = await invoke('import_user_data')
    if (result.success) {
      showMessage('数据导入成功')
    }
  } catch (error) {
    console.error('导入数据失败:', error)
    showMessage(`导入失败: ${error}`)
  }
}

// 创建备份函数
const createBackup = async () => {
  try {
    // const backupPath = await invoke('create_backup')
    showMessage(`备份已创建: ${backupPath}`)
  } catch (error) {
    console.error('创建备份失败:', error)
    showMessage(`备份失败: ${error}`)
  }
}

// 辅助函数
const getAIServiceName = (service) => {
  const serviceMap = {
    'openai': 'OpenAI',
    'claude': 'Claude', 
    'gemini': 'Gemini',
    'deepseek': 'DeepSeek',
    'custom': '自定义'
  }
  return serviceMap[service] || service
}

const getBackupFrequencyName = (frequency) => {
  const frequencyMap = {
    'daily': '每天',
    'weekly': '每周',
    'monthly': '每月'
  }
  return frequencyMap[frequency] || frequency
}

// 云端同步相关函数
// 格式化时间显示
const formatTime = (timestamp) => {
  if (!timestamp) return '';
  const date = new Date(timestamp);
  return `${date.getFullYear()}-${(date.getMonth() + 1).toString().padStart(2, '0')}-${date.getDate().toString().padStart(2, '0')} ${date.getHours().toString().padStart(2, '0')}:${date.getMinutes().toString().padStart(2, '0')}`;
}

// 手动同步
const manualSync = async () => {
  if (isSyncing.value) return;
  
  isSyncing.value = true;
  try {
    // 调用同步API
    //await invoke('force_cloud_sync');
    lastSyncStatus.value = 'success';
    lastSyncTime.value = Date.now();
    
    // 保存同步时间到本地存储
    localStorage.setItem('lastSyncTime', lastSyncTime.value);
    showMessage('同步成功');
  } catch (error) {
    lastSyncStatus.value = 'error';
    console.error('同步失败:', error);
    showMessage(`同步失败: ${error}`);
  } finally {
    isSyncing.value = false;
  }
}

// 立即同步
const syncNow = async () => {
  try {
    showMessage('正在同步...')
    //await invoke('force_cloud_sync')
    showMessage('云端同步完成')
  } catch (error) {
    console.error('同步失败:', error)
    showMessage(`同步失败: ${error}`)
  }
}

// 查看同步状态
const checkSyncStatus = async () => {
  try {
    //const status = await invoke('get_sync_status')
    showMessage(`同步状态: ${status.lastSync ? `最后同步 ${formatTime(status.lastSync)}` : '从未同步'}`)
  } catch (error) {
    console.error('获取同步状态失败:', error)
    showMessage(`获取状态失败: ${error}`)
  }
}

// 更换头像
const changeAvatar = async () => {
  try {
    // const filePath = await invoke('select_avatar_file')
    if (filePath) {
      await invoke('upload_user_avatar', { filePath })
      showMessage('头像更换成功')
    }
  } catch (error) {
    console.error('更换头像失败:', error)
    showMessage(`更换失败: ${error}`)
  }
}

// 修改密码
const changePassword = async () => {
  try {
    // 这里应该打开密码修改模态框
    // const result = await invoke('open_change_password_dialog')
    if (result.success) {
      showMessage('密码修改成功')
    }
  } catch (error) {
    console.error('修改密码失败:', error)
    showMessage(`修改失败: ${error}`)
  }
}

// 删除账户
const deleteAccount = async () => {
  if (confirm('确定要删除账户吗？此操作将永久删除所有数据且不可恢复！')) {
    try {
      // await invoke('delete_user_account')
      showMessage('账户已删除')
      router.push('/')
    } catch (error) {
      console.error('删除账户失败:', error)
      showMessage(`删除失败: ${error}`)
    }
  }
}
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
}

.shortcut-input:hover {
  border-color: #3498db;
  background: #f8f9fa;
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
  z-index: 1000;
  animation: slideUp 0.3s ease;
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