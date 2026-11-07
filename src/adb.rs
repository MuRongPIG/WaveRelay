use std::{
    io::{self, Read, Write},
    net::{Ipv4Addr, SocketAddr, TcpStream},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::atomic::{AtomicBool, Ordering},
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result, bail, ensure};

use crate::protocol::AudioQuality;

pub const SERVER_VERSION: &str = "3.3.4";
const POLL: Duration = Duration::from_millis(150);
const TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug)]
pub struct Device {
    pub serial: String,
    pub state: String,
}

#[derive(Clone)]
pub struct Adb {
    path: PathBuf,
}

impl Adb {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn devices(&self) -> Result<Vec<Device>> {
        let output = self.run(None, &["devices", "-l"])?;
        Ok(parse_devices(&output))
    }

    pub fn wireless(&self, address: &str, code: Option<&str>) -> Result<String> {
        let address: SocketAddr = address
            .trim()
            .parse()
            .context("请输入无线调试页面显示的 IP:端口")?;
        ensure!(address.port() != 0, "端口必须在 1–65535 之间");
        let address = address.to_string();
        if let Some(code) = code {
            ensure!(
                code.len() == 6 && code.bytes().all(|byte| byte.is_ascii_digit()),
                "请输入六位数字配对码"
            );
        }

        let mut command = self.command(None);
        command
            .args([if code.is_some() { "pair" } else { "connect" }, &address])
            .stdin(if code.is_some() {
                Stdio::piped()
            } else {
                Stdio::null()
            })
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let mut child = command
            .spawn()
            .with_context(|| format!("无法运行 ADB ({})", self.path.display()))?;
        let result = (|| {
            if let Some(code) = code {
                // Pass the pairing code through stdin so it is absent from process arguments.
                let mut stdin = child.stdin.take().context("无法向 ADB 输入配对码")?;
                writeln!(stdin, "{code}").context("无法向 ADB 输入配对码")?;
            }
            let start = Instant::now();
            while child.try_wait()?.is_none() {
                ensure!(
                    start.elapsed() < Duration::from_secs(20),
                    "ADB 无线操作超时"
                );
                thread::sleep(POLL);
            }
            Ok(())
        })();
        if let Err(error) = result {
            let _ = child.kill();
            let _ = child.wait();
            return Err(error);
        }
        let output = child.wait_with_output()?;
        let reply = String::from_utf8_lossy(&output.stdout);
        if code.is_some() {
            ensure!(
                output.status.success() && reply.contains("Successfully paired to"),
                "配对失败。请确认配对地址和配对码有效，然后重试"
            );
        } else {
            let connected = reply.lines().any(|line| {
                line.starts_with("connected to ") || line.starts_with("already connected to ")
            });
            ensure!(
                output.status.success() && connected,
                "连接失败。请检查无线调试主页的连接地址、配对状态和网络"
            );
        }
        Ok(address)
    }

    pub fn select(&self, serial: Option<&str>) -> Result<String> {
        let devices = self.devices()?;
        if let Some(serial) = serial {
            let device = devices
                .iter()
                .find(|device| device.serial == serial)
                .with_context(|| format!("ADB device {serial} not found"))?;
            ensure!(
                device.state == "device",
                "ADB device {serial} is {}; check debugging authorization",
                device.state
            );
            return Ok(device.serial.clone());
        }

        let mut ready = devices.iter().filter(|device| device.state == "device");
        let first = ready.next().context("No authorized Android device. Run waverelay devices and check USB or wireless debugging.")?;
        ensure!(
            ready.next().is_none(),
            "Multiple devices are ready; choose one with --serial"
        );
        Ok(first.serial.clone())
    }

    fn command(&self, serial: Option<&str>) -> Command {
        let mut command = Command::new(&self.path);
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x0800_0000;
            command.creation_flags(CREATE_NO_WINDOW);
        }
        if let Some(serial) = serial {
            command.args(["-s", serial]);
        }
        command
    }

    fn run(&self, serial: Option<&str>, args: &[&str]) -> Result<String> {
        self.output(self.command(serial).args(args), &args.join(" "))
    }

    fn output(&self, command: &mut Command, operation: &str) -> Result<String> {
        let output = command
            .output()
            .with_context(|| format!("Could not run ADB at {}", self.path.display()))?;
        ensure!(
            output.status.success(),
            "adb {operation} failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    }
}

fn parse_devices(output: &str) -> Vec<Device> {
    output
        .lines()
        .filter_map(|line| {
            let mut fields = line.split_whitespace();
            let serial = fields.next()?;
            let state = fields.next()?;
            if matches!(serial, "List" | "*") {
                return None;
            }
            Some(Device {
                serial: serial.into(),
                state: state.into(),
            })
        })
        .collect()
}

pub struct Session<'a> {
    adb: &'a Adb,
    serial: &'a str,
    remote_file: String,
    forward: Option<u16>,
    server: Option<Child>,
}

impl<'a> Session<'a> {
    pub fn start(
        adb: &'a Adb,
        serial: &'a str,
        server: &Path,
        quality: AudioQuality,
        stop: &AtomicBool,
    ) -> Result<Self> {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
        let pid = std::process::id();
        let scid = ((timestamp as u32) ^ pid.rotate_left(12)) & 0x7fff_ffff;
        let remote_file = format!("/data/local/tmp/waverelay-{pid}-{timestamp}.jar");
        let socket = format!("localabstract:scrcpy_{scid:08x}");
        let mut session = Self {
            adb,
            serial,
            remote_file,
            forward: None,
            server: None,
        };

        adb.output(
            adb.command(Some(serial))
                .arg("push")
                .arg(server)
                .arg(&session.remote_file),
            "push audio server",
        )?;
        check_stop(stop)?;

        let port = adb.run(Some(serial), &["forward", "tcp:0", &socket])?;
        let port: u16 = port
            .trim()
            .parse()
            .context("ADB returned an invalid forwarded port")?;
        ensure!(port != 0, "ADB did not allocate a forwarded port");
        session.forward = Some(port);
        check_stop(stop)?;

        let classpath = format!("CLASSPATH={}", session.remote_file);
        let scid_arg = format!("scid={scid:08x}");
        let args = [
            "shell",
            "-T",
            &classpath,
            "exec",
            "app_process",
            "/",
            "com.genymobile.scrcpy.Server",
            SERVER_VERSION,
            &scid_arg,
            "tunnel_forward=true",
            "video=false",
            "audio=true",
            "control=false",
            "audio_source=output",
            "send_device_meta=false",
            "send_dummy_byte=true",
            "send_codec_meta=true",
            "send_frame_meta=true",
            "cleanup=false",
            "power_on=false",
        ];

        session.server = Some(
            adb.command(Some(serial))
                .args(args)
                .args(quality.server_args())
                .stdin(Stdio::null())
                .spawn()
                .context("Could not launch scrcpy on Android")?,
        );
        Ok(session)
    }

    pub fn open<'stop>(&mut self, stop: &'stop AtomicBool) -> Result<SocketReader<'stop>> {
        let port = self.forward.context("ADB forward is not active")?;
        let address = (Ipv4Addr::LOCALHOST, port).into();
        let start = Instant::now();
        while start.elapsed() < TIMEOUT {
            check_stop(stop)?;
            if let Some(status) = self.server.as_mut().unwrap().try_wait()? {
                bail!("Android audio server exited before connecting ({status})");
            }
            if let Ok(stream) = TcpStream::connect_timeout(&address, POLL) {
                stream.set_read_timeout(Some(POLL))?;
                let mut reader = SocketReader {
                    stream,
                    stop,
                    last_read: start,
                };
                let mut dummy = [0; 1];
                match reader.read(&mut dummy) {
                    Ok(1) if dummy[0] == 0 => return Ok(reader),
                    Ok(1) => bail!("Unexpected scrcpy tunnel handshake"),
                    Ok(_) => (),
                    Err(err)
                        if matches!(
                            err.kind(),
                            io::ErrorKind::ConnectionReset | io::ErrorKind::ConnectionAborted
                        ) => {}
                    Err(err) => return Err(err.into()),
                }
            }
            thread::sleep(POLL);
        }
        bail!("Timed out waiting for the Android audio socket")
    }
}

impl Drop for Session<'_> {
    fn drop(&mut self) {
        if let Some(server) = &mut self.server {
            // The caller drops the socket first, giving Android a chance to close the capture.
            let deadline = Instant::now() + Duration::from_millis(500);
            while matches!(server.try_wait(), Ok(None)) && Instant::now() < deadline {
                thread::sleep(Duration::from_millis(25));
            }
            let _ = server.kill();
            let _ = server.wait();
        }
        if let Some(port) = self.forward
            && let Err(error) = self.adb.run(
                Some(self.serial),
                &["forward", "--remove", &format!("tcp:{port}")],
            )
        {
            eprintln!("Could not remove WaveRelay's ADB forward: {error:#}");
        }
        if let Err(error) = self
            .adb
            .run(Some(self.serial), &["shell", "rm", "-f", &self.remote_file])
        {
            eprintln!("Could not remove WaveRelay's temporary Android server: {error:#}");
        }
    }
}

fn check_stop(stop: &AtomicBool) -> io::Result<()> {
    if stop.load(Ordering::Relaxed) {
        // Read::read_exact retries Interrupted, so cancellation uses a distinct error.
        return Err(io::Error::other("Relay stopped"));
    }
    Ok(())
}

pub struct SocketReader<'a> {
    stream: TcpStream,
    stop: &'a AtomicBool,
    last_read: Instant,
}

impl Read for SocketReader<'_> {
    fn read(&mut self, data: &mut [u8]) -> io::Result<usize> {
        loop {
            check_stop(self.stop)?;
            match self.stream.read(data) {
                Ok(size) => {
                    self.last_read = Instant::now();
                    return Ok(size);
                }
                Err(err)
                    if matches!(
                        err.kind(),
                        io::ErrorKind::TimedOut | io::ErrorKind::WouldBlock
                    ) && self.last_read.elapsed() < TIMEOUT => {}
                Err(err) => return Err(err),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_usb_wifi_and_unavailable_devices() {
        let devices = parse_devices(
            "List of devices attached\r\nUSB123\tunauthorized\r\n192.168.1.2:1234 device product:test\r\nUSB456 offline\r\n",
        );
        assert_eq!(devices.len(), 3);
        assert_eq!(devices[0].state, "unauthorized");
        assert_eq!(devices[1].serial, "192.168.1.2:1234");
        assert_eq!(devices[1].state, "device");
        assert_eq!(devices[2].state, "offline");
    }
}
