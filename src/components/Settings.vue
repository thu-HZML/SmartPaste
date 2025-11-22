<script setup>
import { reactive, onMounted } from 'vue';
import { useRouter } from 'vue-router'

const data = reactive({});

const router = useRouter()

const goBack = () => {
  router.back()
}
</script>

<template>
  <div class="flex-col page">
    <button @click="goBack">← 返回</button>
    <div class="flex-row justify-between items-center section">
      <div class="flex-row items-center">
        <img
          class="shrink-0 image"
          src="https://ide.code.fun/api/image?token=6900babc9520a30011f4cfff&name=d13956be8b29d3a77edf883ca43b3333.png"
        />
        <span class="text ml-17">Smartpaste</span>
      </div>
      <img
        class="image_2"
        src="https://ide.code.fun/api/image?token=6900babc9520a30011f4cfff&name=2445f3532f8a96be1689811c82ec039d.png"
      />
    </div>
    <div class="flex-row items-center">
      <div class="flex-col shrink-0 section_2">
        <div class="flex-row items-center self-stretch group_2">
          <img
            class="image"
            src="https://ide.code.fun/api/image?token=6900babc9520a30011f4cfff&name=c93fc0d3e7e98e475f80b1418f29144b.png"
          />
          <span class="ml-26 font text_2">历史记录</span>
        </div>
        <div class="flex-row items-center self-stretch group_4">
          <img
            class="image"
            src="https://ide.code.fun/api/image?token=6900babc9520a30011f4cfff&name=016385210c93bad50e2c46053dbb3cee.png"
          />
          <span class="font text_4 ml-25">收藏夹</span>
        </div>
        <div class="flex-row items-center self-stretch group_6">
          <img
            class="image"
            src="https://ide.code.fun/api/image?token=6900babc9520a30011f4cfff&name=9f467414a3934476654b25dd66dc09eb.png"
          />
          <span class="font text_6 ml-19">分类</span>
        </div>
        <div class="flex-row items-center self-stretch group_8">
          <img
            class="image"
            src="https://ide.code.fun/api/image?token=6900babc9520a30011f4cfff&name=7201cba06a56a9aecdf081772f8ee616.png"
          />
          <span class="font ml-15">群组</span>
        </div>
        <img
          class="self-start image image_6"
          src="https://ide.code.fun/api/image?token=6900babc9520a30011f4cfff&name=85e6c4051832477789a9c724127ba087.png"
        />
      </div>
      <div class="ml-78 flex-col flex-1 group">
        <!-- 文本内容项 -->
        <div class="flex-col section_3">
          <div class="flex-row justify-evenly items-center self-start section_4">
            <img
              class="image_3"
              src="https://ide.code.fun/api/image?token=6900babc9520a30011f4cfff&name=146e11466234cbabec75cb8ce961906f.png"
            />
            <span class="font_2">TXT</span>
          </div>
          <div class="self-stretch group_3 mt-15">
            <span class="font_3">齿轮是指轮缘上有齿，能连续</span>
            <span class="font_3 text_3">啮合</span>
            <span class="font_3">
              传递运动和动力的机械元件。齿轮在传动中的应用很早就出现了。19世纪末，展成切齿法的原理及利用此原理切齿的
            </span>
            <span class="font_3 text_3">专用机床</span>
            <span class="font_3">与</span>
            <span class="font_3 text_3">刀具</span>
            <span class="font_3">的相继出现，随着生产的发展，齿轮运转的平稳性受到重视。</span>
          </div>
          <div class="flex-row justify-end self-stretch mt-15">
            <div class="flex-row group_5">
              <span class="font_4 text_5">删除</span>
              <img
                class="ml-6 image_3"
                src="https://ide.code.fun/api/image?token=6900babc9520a30011f4cfff&name=6583c1c038f107ebe85b354a3088a549.png"
              />
            </div>
            <div class="flex-row section_5">
              <img
                class="shrink-0 image_3 image_4"
                src="https://ide.code.fun/api/image?token=6900babc9520a30011f4cfff&name=e969f76c0b9bb870e0965147bab86b7e.png"
              />
              <span class="ml-8 font_5">收藏</span>
            </div>
          </div>
        </div>

        <!-- HTML内容项 -->
        <div class="mt-34 flex-col section_6">
          <div class="flex-col self-start group_7">
            <div class="flex-row justify-evenly items-center self-start section_7">
              <img
                class="image_3"
                src="https://ide.code.fun/api/image?token=6900babc9520a30011f4cfff&name=a483d42e6a72131cd5bc4083ee4111e8.png"
              />
              <span class="font_2">HTML</span>
            </div>
            <div class="mt-10 self-stretch font_3">
              &lt;title&gt;个人作品集 | 现代设计&lt;/title&gt;<br>
              &lt;link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.4.0/css/all.min.css" /&gt;<br>
              &lt;style&gt;<br>
              &nbsp;&nbsp;/* CSS代码示例 */<br>
              &lt;/style&gt;
            </div>
          </div>
          <div class="mt-12 flex-row justify-end self-stretch">
            <div class="flex-row group_5">
              <span class="font_4 text_7">删除</span>
              <img class="ml-6 image_3" src="https://ide.code.fun/api/image?token=6900babc9520a30011f4cfff&name=6583c1c038f107ebe85b354a3088a549.png"/>
            </div>
            <div class="flex-row section_5">
              <img class="shrink-0 image_3 image_4" src="https://ide.code.fun/api/image?token=6900babc9520a30011f4cfff&name=e969f76c0b9bb870e0965147bab86b7e.png"/>
              <span class="ml-8 font_5">收藏</span>
            </div>
          </div>
        </div>

        <!-- 链接内容项 -->
        <div class="mt-34 flex-col section_8">
          <div class="flex-col items-start">
            <div class="flex-row justify-evenly items-center section_4">
              <img class="image_3" src="https://ide.code.fun/api/image?token=6900babc9520a30011f4cfff&name=fea0fed74ff60618916e7e9b6e207830.png"/>
              <span class="font_2 text_8">Link</span>
            </div>
            <span class="font_3 text_9 mt-15">https://learn.tsinghua.edu.cn/f/wlxt/index/course/student/</span>
          </div>
          <div class="mt-32 flex-row justify-end">
            <div class="flex-row group_5">
              <span class="font_4 text_10">删除</span>
              <img class="ml-6 image_3" src="https://ide.code.fun/api/image?token=6900babc9520a30011f4cfff&name=6583c1c038f107ebe85b354a3088a549.png"/>
            </div>
            <div class="flex-row section_5">
              <img class="shrink-0 image_3 image_4" src="https://ide.code.fun/api/image?token=6900babc9520a30011f4cfff&name=e969f76c0b9bb870e0965147bab86b7e.png"/>
              <span class="ml-8 font_5">收藏</span>
            </div>
          </div>
        </div>

        <!-- 图片内容项 -->
        <div class="mt-34 flex-row justify-between items-end section_6">
          <div class="flex-col group_9">
            <div class="flex-row items-center self-start section_9">
              <img class="shrink-0 image_3" src="https://ide.code.fun/api/image?token=6900babc9520a30011f4cfff&name=13b8e01b870e5d9384842314ad8f5b43.png"/>
              <span class="font_2 text_11 ml-3">Picture</span>
            </div>
            <img class="shrink-0 self-end image_5" src="https://ide.code.fun/api/image?token=6900babc9520a30011f4cfff&name=e66221c416ab8e65450fecd31272aae9.png"/>
          </div>
          <div class="flex-row">
            <div class="flex-row group_5">
              <span class="font_4 text_7">删除</span>
              <img class="ml-6 image_3" src="https://ide.code.fun/api/image?token=6900babc9520a30011f4cfff&name=6583c1c038f107ebe85b354a3088a549.png"/>
            </div>
            <div class="flex-row justify-center section_10">
              <img class="image_3" src="https://ide.code.fun/api/image?token=6900babc9520a30011f4cfff&name=e969f76c0b9bb870e0965147bab86b7e.png"/>
              <span class="ml-8 font_5">收藏</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style>
/************************************************************
** 请将全局样式拷贝到项目的全局 CSS 文件或者当前页面的顶部 **
** 否则页面将无法正常显示                                  **
************************************************************/

html {
  font-size: 16px;
}

body {
  margin: 0;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Roboto', 'Oxygen', 'Ubuntu', 'Cantarell', 'Fira Sans',
    'Droid Sans', 'Helvetica Neue', 'Microsoft Yahei', sans-serif;
}

body * {
  box-sizing: border-box;
  flex-shrink: 0;
}

.flex-row {
  display: flex;
  flex-direction: row;
}

.flex-col {
  display: flex;
  flex-direction: column;
}

.justify-start {
  justify-content: flex-start;
}

.justify-end {
  justify-content: flex-end;
}

.justify-center {
  justify-content: center;
}

.justify-between {
  justify-content: space-between;
}

.justify-around {
  justify-content: space-around;
}

.justify-evenly {
  justify-content: space-evenly;
}

.items-start {
  align-items: flex-start;
}

.items-end {
  align-items: flex-end;
}

.items-center {
  align-items: center;
}

.items-baseline {
  align-items: baseline;
}

.items-stretch {
  align-items: stretch;
}

.self-start {
  align-self: flex-start;
}

.self-end {
  align-self: flex-end;
}

.self-center {
  align-self: center;
}

.self-baseline {
  align-self: baseline;
}

.self-stretch {
  align-self: stretch;
}

.flex-1 {
  flex: 1 1 0%;
}

.flex-auto {
  flex: 1 1 auto;
}

.grow {
  flex-grow: 1;
}

.grow-0 {
  flex-grow: 0;
}

.shrink {
  flex-shrink: 1;
}

.shrink-0 {
  flex-shrink: 0;
}

.relative {
  position: relative;
}

.ml-2 {
  margin-left: 0.13rem;
}

.mt-2 {
  margin-top: 0.13rem;
}

.ml-4 {
  margin-left: 0.25rem;
}

.mt-4 {
  margin-top: 0.25rem;
}

.ml-6 {
  margin-left: 0.38rem;
}

.mt-6 {
  margin-top: 0.38rem;
}

.ml-8 {
  margin-left: 0.5rem;
}

.mt-8 {
  margin-top: 0.5rem;
}

.ml-10 {
  margin-left: 0.63rem;
}

.mt-10 {
  margin-top: 0.63rem;
}

.ml-12 {
  margin-left: 0.75rem;
}

.mt-12 {
  margin-top: 0.75rem;
}

.ml-14 {
  margin-left: 0.88rem;
}

.mt-14 {
  margin-top: 0.88rem;
}

.ml-16 {
  margin-left: 1rem;
}

.mt-16 {
  margin-top: 1rem;
}

.ml-18 {
  margin-left: 1.13rem;
}

.mt-18 {
  margin-top: 1.13rem;
}

.ml-20 {
  margin-left: 1.25rem;
}

.mt-20 {
  margin-top: 1.25rem;
}

.ml-22 {
  margin-left: 1.38rem;
}

.mt-22 {
  margin-top: 1.38rem;
}

.ml-24 {
  margin-left: 1.5rem;
}

.mt-24 {
  margin-top: 1.5rem;
}

.ml-26 {
  margin-left: 1.63rem;
}

.mt-26 {
  margin-top: 1.63rem;
}

.ml-28 {
  margin-left: 1.75rem;
}

.mt-28 {
  margin-top: 1.75rem;
}

.ml-30 {
  margin-left: 1.88rem;
}

.mt-30 {
  margin-top: 1.88rem;
}

.ml-32 {
  margin-left: 2rem;
}

.mt-32 {
  margin-top: 2rem;
}

.ml-34 {
  margin-left: 2.13rem;
}

.mt-34 {
  margin-top: 2.13rem;
}

.ml-36 {
  margin-left: 2.25rem;
}

.mt-36 {
  margin-top: 2.25rem;
}

.ml-38 {
  margin-left: 2.38rem;
}

.mt-38 {
  margin-top: 2.38rem;
}

.ml-40 {
  margin-left: 2.5rem;
}

.mt-40 {
  margin-top: 2.5rem;
}

.ml-42 {
  margin-left: 2.63rem;
}

.mt-42 {
  margin-top: 2.63rem;
}

.ml-44 {
  margin-left: 2.75rem;
}

.mt-44 {
  margin-top: 2.75rem;
}

.ml-46 {
  margin-left: 2.88rem;
}

.mt-46 {
  margin-top: 2.88rem;
}

.ml-48 {
  margin-left: 3rem;
}

.mt-48 {
  margin-top: 3rem;
}

.ml-50 {
  margin-left: 3.13rem;
}

.mt-50 {
  margin-top: 3.13rem;
}

.ml-52 {
  margin-left: 3.25rem;
}

.mt-52 {
  margin-top: 3.25rem;
}

.ml-54 {
  margin-left: 3.38rem;
}

.mt-54 {
  margin-top: 3.38rem;
}

.ml-56 {
  margin-left: 3.5rem;
}

.mt-56 {
  margin-top: 3.5rem;
}

.ml-58 {
  margin-left: 3.63rem;
}

.mt-58 {
  margin-top: 3.63rem;
}

.ml-60 {
  margin-left: 3.75rem;
}

.mt-60 {
  margin-top: 3.75rem;
}

.ml-62 {
  margin-left: 3.88rem;
}

.mt-62 {
  margin-top: 3.88rem;
}

.ml-64 {
  margin-left: 4rem;
}

.mt-64 {
  margin-top: 4rem;
}

.ml-66 {
  margin-left: 4.13rem;
}

.mt-66 {
  margin-top: 4.13rem;
}

.ml-68 {
  margin-left: 4.25rem;
}

.mt-68 {
  margin-top: 4.25rem;
}

.ml-70 {
  margin-left: 4.38rem;
}

.mt-70 {
  margin-top: 4.38rem;
}

.ml-72 {
  margin-left: 4.5rem;
}

.mt-72 {
  margin-top: 4.5rem;
}

.ml-74 {
  margin-left: 4.63rem;
}

.mt-74 {
  margin-top: 4.63rem;
}

.ml-76 {
  margin-left: 4.75rem;
}

.mt-76 {
  margin-top: 4.75rem;
}

.ml-78 {
  margin-left: 4.88rem;
}

.mt-78 {
  margin-top: 4.88rem;
}

.ml-80 {
  margin-left: 5rem;
}

.mt-80 {
  margin-top: 5rem;
}

.ml-82 {
  margin-left: 5.13rem;
}

.mt-82 {
  margin-top: 5.13rem;
}

.ml-84 {
  margin-left: 5.25rem;
}

.mt-84 {
  margin-top: 5.25rem;
}

.ml-86 {
  margin-left: 5.38rem;
}

.mt-86 {
  margin-top: 5.38rem;
}

.ml-88 {
  margin-left: 5.5rem;
}

.mt-88 {
  margin-top: 5.5rem;
}

.ml-90 {
  margin-left: 5.63rem;
}

.mt-90 {
  margin-top: 5.63rem;
}

.ml-92 {
  margin-left: 5.75rem;
}

.mt-92 {
  margin-top: 5.75rem;
}

.ml-94 {
  margin-left: 5.88rem;
}

.mt-94 {
  margin-top: 5.88rem;
}

.ml-96 {
  margin-left: 6rem;
}

.mt-96 {
  margin-top: 6rem;
}

.ml-98 {
  margin-left: 6.13rem;
}

.mt-98 {
  margin-top: 6.13rem;
}

.ml-100 {
  margin-left: 6.25rem;
}

.mt-100 {
  margin-top: 6.25rem;
}

/* 从 MacBook_Pro_16_3.vue 合并的样式 */
.ml-17 {
  margin-left: 1.06rem;
}
.ml-25 {
  margin-left: 1.56rem;
}
.ml-19 {
  margin-left: 1.19rem;
}
.ml-15 {
  margin-left: 0.94rem;
}
.mt-15 {
  margin-top: 0.94rem;
}
.ml-3 {
  margin-left: 0.19rem;
}
.page {
  background-color: rgb(255, 255, 255);
  border-radius: 0.5rem;
  width: 100%;
  overflow-y: auto;
  overflow-x: hidden;
  height: 100%;
}
.section {
  padding: 1.5rem 1.88rem 1.25rem 2.38rem;
  background-color: rgb(30, 146, 247);
}
.image {
  width: 3rem;
  height: 3rem;
}
.text {
  color: rgb(0, 0, 0);
  font-size: 2rem;
  font-family: Inter;
  line-height: 1.88rem;
  text-shadow: 0 0.063rem #000000, 0 -0.063rem #000000, 0.063rem 0 #000000, -0.063rem 0 rgb(0, 0, 0);
}
.image_2 {
  width: 3.44rem;
  height: 3.25rem;
}
.section_2 {
  padding: 3.25rem 1rem 1.38rem;
  background-color: rgb(230, 243, 248);
  width: 17.27rem;
  height: 55.25rem;
}
.group_2 {
  padding: 0 1.5rem;
}
.font {
  font-size: 1.5rem;
  font-family: Inter;
  line-height: 1.38rem;
  color: rgb(0, 0, 0);
}
.text_2 {
  line-height: 1.38rem;
}
.group_4 {
  margin-top: 2.25rem;
  padding: 0 1.63rem;
}
.text_4 {
  line-height: 1.38rem;
}
.group_6 {
  margin-top: 2.19rem;
  padding: 0 1.88rem;
}
.text_6 {
  line-height: 1.39rem;
}
.group_8 {
  margin-top: 2.13rem;
  padding: 0 2rem;
}
.image_6 {
  margin-top: 29.06rem;
}
.group {
  margin-right: 1.88rem;
}
.section_3 {
  padding: 1.25rem 0.75rem 0.88rem 1.63rem;
  background-color: rgb(230, 244, 248);
}
.section_4 {
  padding: 0.5rem;
  background-color: rgb(44, 44, 44);
  border-radius: 0.5rem;
  width: 4.06rem;
}
.image_3 {
  width: 1rem;
  height: 1rem;
}
.font_2 {
  font-size: 1rem;
  font-family: Inter;
  line-height: 0.73rem;
  color: rgb(245, 245, 245);
}
.group_3 {
  line-height: 1.19rem;
}
.font_3 {
  font-size: 1rem;
  font-family: Inter;
  line-height: 1.19rem;
  color: rgb(0, 0, 0);
}
.text_3 {
  text-decoration: underline;
}
.group_5 {
  padding: 0.75rem 0;
  border-radius: 0.5rem;
  overflow: hidden;
  width: 7.5rem;
  height: 2.5rem;
}
.font_4 {
  font-size: 1rem;
  font-family: Inter;
  line-height: 0.92rem;
  color: rgb(48, 48, 48);
}
.text_5 {
  margin-left: 2rem;
  line-height: 0.92rem;
}
.section_5 {
  padding: 0.75rem 1.75rem;
  background-color: rgb(44, 44, 44);
  border-radius: 0.5rem;
  overflow: hidden;
  width: 7.5rem;
  height: 2.5rem;
  border-left: solid 0.063rem rgb(44, 44, 44);
  border-right: solid 0.063rem rgb(44, 44, 44);
  border-top: solid 0.063rem rgb(44, 44, 44);
  border-bottom: solid 0.063rem rgb(44, 44, 44);
}
.image_4 {
  margin-left: 0.25rem;
}
.font_5 {
  font-size: 1rem;
  font-family: Inter;
  line-height: 0.92rem;
  color: rgb(245, 245, 245);
}
.section_6 {
  padding: 0.88rem 0.75rem 0.88rem 1.63rem;
  background-color: rgb(230, 244, 248);
}
.group_7 {
  width: 50.48rem;
}
.section_7 {
  padding: 0.5rem;
  background-color: rgb(44, 44, 44);
  border-radius: 0.5rem;
  width: 5rem;
}
.text_7 {
  margin-left: 2rem;
  line-height: 0.92rem;
}
.section_8 {
  padding: 1.88rem 0.75rem 0.88rem 1.63rem;
  background-color: rgb(230, 244, 248);
}
.text_8 {
  line-height: 0.75rem;
}
.text_9 {
  margin-left: 0.13rem;
  line-height: 0.98rem;
}
.text_10 {
  margin-left: 2rem;
  line-height: 0.92rem;
}
.group_9 {
  margin-bottom: 0.31rem;
  width: 14.38rem;
  height: 9rem;
}
.section_9 {
  padding: 0.5rem 0.38rem 0.5rem 0.5rem;
  background-color: rgb(44, 44, 44);
  border-radius: 0.5rem;
}
.text_11 {
  line-height: 0.76rem;
}
.image_5 {
  margin-top: -0.5rem;
  width: 8.06rem;
  height: 7.5rem;
}
.section_10 {
  padding: 0.75rem 0;
  background-color: rgb(44, 44, 44);
  border-radius: 0.5rem;
  overflow: hidden;
  width: 7.5rem;
  height: 2.5rem;
  border-left: solid 0.063rem rgb(44, 44, 44);
  border-right: solid 0.063rem rgb(44, 44, 44);
  border-top: solid 0.063rem rgb(44, 44, 44);
  border-bottom: solid 0.063rem rgb(44, 44, 44);
}
</style>