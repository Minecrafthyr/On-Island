# On Island

This is a island survival game, functionalities are incomplete and basically not working now.

Contributes are welcome.

Currently we only have Simplified Chinese translation, without English.

---

这是一个孤岛生存游戏。目前功能不完备，且基本上没法运行。

欢迎贡献。

目前只有中文翻译，没有英文。

## 技术细节/计划

使用Rust语言。项目具有相当实验性，使用了nightly的const相关特性开关。

目前使用常量/static函数接口，不支持动态数据加载，且为单线程。

游戏时间精确至us，1ms为一刻。在行动确定后时间才会流逝。

设计为尽可能真实，并将在稳定后添加超现实内容。
