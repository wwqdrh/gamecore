## 构建软连接

```bash
ln -s /Applications/Godot.app/Contents/MacOS/Godot /usr/local/bin/godot
```

## 单元测试

```bash
godot --headless --path . -s res://test/test_runner.gd
```