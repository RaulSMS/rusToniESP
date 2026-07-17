# Board Configuration

The target board profile is determined **strictly by your active compilation target**. There is no need to pass manual environment variables during execution.

To select or switch the board layout, edit the master switchboard at the top of your `.cargo/config.toml` file:

* **ESP32-S3:** Set `target = "xtensa-esp32s3-espidf"`
* **ESP32 (WROOM/WROVER):** Set `target = "xtensa-esp32-espidf"`

### Project Build Command
Once the target is uncommented in `.cargo/config.toml`, simply run:

```bash
cargo build