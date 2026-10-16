class_name XiuUILayers
## CanvasLayer 层级常量表 —— demo 全部 UI 层级的唯一出处
##
## 嵌套 CanvasLayer 的 layer 是全局排序的（不随父节点），各层号必须统一
## 规划，严禁散落魔法数字（历史坑：默认 layer=1 的对话框被 mainhud
## 底部道具栏遮住）。新增层级先来这里占号。
##
## 层号分配：
##   1        主 HUD（mainhud UILayer）
##   2        通用对话层（dialog_layer）
##   3        对话框（dialog_box）
##   10       全屏层段（探索 / 日历 / 秘境等覆盖层）
##   100      演出 UI 保留段（rust 默认演出对话框 MOVIE_DIALOG_LAYER 同值，
##            演出期恒最顶）

## 主 HUD（mainhud UILayer）
const BASE_UI := 1
## 通用对话层
const DIALOG := 2
## 对话框
const DIALOG_BOX := 3
## 全屏覆盖层段（探索 / 日历 / 秘境）
const FULLSCREEN := 10
## 演出 UI 保留段（GdMovieDialogBox / 自定义演出 UI）
const MOVIE := 100
