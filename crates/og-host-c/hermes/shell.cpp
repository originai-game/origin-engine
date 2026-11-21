// The desktop shell: Hermes running a game's bytecode on the C ABI host
// (og-host-c). It registers every entry of og_c_symbols() as a function of
// `globalThis.__og_host`, turns OgValues into JavaScript values and back,
// and gives the game's dispatcher to the engine. A bundle that opens the
// window (packages/engine/src/hermes/shell.ts) leaves `__og_frame`, and the
// shell runs the display loop: pump the window, call the frame with the
// time now, wait until the time it answers (when the engine's frame cap
// next draws), until the window closes; then it prints `__og_report()` and
// calls `__og_close()`. A bundle without one (the
// game-frame benchmark, bench/game-frame/hermes.ts) runs once.
//
//   og-hermes <bundle.hbc> [key=value ...]
//   og-hermes                  (a package: game.hbc beside it, or in ../Resources)
//
// Each key=value becomes `globalThis.__og_env[key]` (key=@file: the file's
// text); a package's shell.env lines are read first, and OG_ROOT and
// OG_PROJECT name its directory, and OG_LOCALE is the OS's locale unless
// given. OG_SHELL_FRAMES=<n> closes after n display frames;
// OG_SHELL_CAPTURE=<file.png> writes the last frame there.

#include <hermes/hermes.h>
#include <jsi/jsi.h>

#include <chrono>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <filesystem>
#include <fstream>
#include <memory>
#include <sstream>
#include <string>
#include <thread>
#include <vector>

#include "og_host.h"

#if defined(__APPLE__)
#include <mach-o/dyld.h>
#elif defined(_WIN32)
#include <windows.h>
#endif

using namespace facebook;

namespace {

/// Memory the engine owns, shown to JavaScript without a copy.
class Borrowed : public jsi::MutableBuffer {
 public:
  Borrowed(uint8_t *data, size_t size) : data_(data), size_(size) {}
  size_t size() const override { return size_; }
  uint8_t *data() override { return data_; }

 private:
  uint8_t *data_;
  size_t size_;
};

/// Bytes JavaScript owns, copied out of an answer.
class Owned : public jsi::MutableBuffer {
 public:
  Owned(const void *data, size_t size)
      : bytes_((const uint8_t *)data, (const uint8_t *)data + size) {}
  size_t size() const override { return bytes_.size(); }
  uint8_t *data() override { return bytes_.data(); }

 private:
  std::vector<uint8_t> bytes_;
};

struct Shell {
  jsi::Runtime *rt = nullptr;
  std::unique_ptr<jsi::Function> dispatcher;
  /// The answer of the last dispatch, kept until the next.
  std::string answer;
  /// The column buffers handed out since they were last retired.
  std::vector<jsi::Value> views;
  OgDispatcher table{};
};

Shell shell;

jsi::Value typed(jsi::Runtime &rt, const char *ctor, jsi::ArrayBuffer buffer) {
  return rt.global().getPropertyAsFunction(rt, ctor).callAsConstructor(rt, buffer);
}

/// A typed array's bytes: its buffer's data at its offset.
std::pair<const uint8_t *, size_t> bytesOf(jsi::Runtime &rt, const jsi::Object &array) {
  auto buffer = array.getPropertyAsObject(rt, "buffer").getArrayBuffer(rt);
  size_t offset = (size_t)array.getProperty(rt, "byteOffset").asNumber();
  size_t length = (size_t)array.getProperty(rt, "byteLength").asNumber();
  return {buffer.data(rt) + offset, length};
}

void dispatchCall(
    void *, uint32_t kind, uint32_t engine, uint32_t id, uint32_t query,
    double tick, double dt, double elapsed, const uint8_t *payload,
    size_t payloadLen, const uint8_t **answer, size_t *answerLen) {
  jsi::Runtime &rt = *shell.rt;
  try {
    jsi::Value r = shell.dispatcher->call(
        rt, (double)kind, (double)engine, (double)id, (double)query, tick, dt,
        elapsed,
        jsi::String::createFromUtf8(rt, payload, payloadLen));
    shell.answer = r.isString() ? r.getString(rt).utf8(rt) : std::string();
  } catch (const jsi::JSError &e) {
    shell.answer = e.getMessage() + "\n" + e.getStack();
    if (shell.answer.empty()) shell.answer = "game code threw";
  }
  *answer = (const uint8_t *)shell.answer.data();
  *answerLen = shell.answer.size();
}

void invalidate(void *, bool moving) {
  if (!moving) return;
  jsi::Runtime &rt = *shell.rt;
  // A kept typed array reads as empty instead of reaching moved memory:
  // the ArrayBuffers over columns are detached (third_party/hermes's
  // IHermesArrayBufferDetach; the runtime's test methods stay off).
  auto *detach = jsi::castInterface<facebook::hermes::IHermesArrayBufferDetach>(&rt);
  for (auto &v : shell.views) detach->detachArrayBuffer(v.getObject(rt).getArrayBuffer(rt));
  shell.views.clear();
}

OgValue toC(jsi::Runtime &rt, const jsi::Value &v, std::vector<std::string> &keep) {
  OgValue c{};
  if (v.isUndefined() || v.isNull()) return c;
  if (v.isBool()) {
    c.tag = OG_BOOL;
    c.num = v.getBool() ? 1 : 0;
  } else if (v.isNumber()) {
    c.tag = OG_NUMBER;
    c.num = v.getNumber();
  } else if (v.isString()) {
    keep.push_back(v.getString(rt).utf8(rt));
    c.tag = OG_STRING;
    c.ptr = keep.back().data();
    c.len = (uint32_t)keep.back().size();
  } else if (v.isObject()) {
    jsi::Object o = v.getObject(rt);
    if (o.isFunction(rt)) {
      shell.dispatcher = std::make_unique<jsi::Function>(o.getFunction(rt));
      shell.table = {nullptr, dispatchCall, invalidate};
      c.tag = OG_FUNCTION;
      c.ptr = &shell.table;
    } else {
      auto [data, len] = bytesOf(rt, o);
      c.tag = OG_BYTES;
      c.ptr = data;
      c.len = (uint32_t)len;
    }
  }
  return c;
}

jsi::Value toJs(jsi::Runtime &rt, const OgValue &c) {
  switch (c.tag) {
    case OG_BOOL:
      return jsi::Value(c.num != 0);
    case OG_NUMBER:
      return jsi::Value(c.num);
    case OG_STRING:
      return jsi::String::createFromUtf8(rt, (const uint8_t *)c.ptr, c.len);
    case OG_BYTES:
      return typed(rt, "Uint8Array",
                   jsi::ArrayBuffer(rt, std::make_shared<Owned>(c.ptr, c.len)));
    case OG_U32_ARRAY:
      return typed(rt, "Uint32Array",
                   jsi::ArrayBuffer(rt, std::make_shared<Owned>(c.ptr, c.len * 4)));
    case OG_VIEWS: {
      auto cols = (const OgColumn *)c.ptr;
      jsi::Array out(rt, c.len);
      for (uint32_t i = 0; i < c.len; i++) {
        jsi::Value b = jsi::ArrayBuffer(
            rt, std::make_shared<Borrowed>(cols[i].ptr, cols[i].len));
        shell.views.emplace_back(rt, b);
        out.setValueAtIndex(rt, i, std::move(b));
      }
      return out;
    }
    case OG_SHARED:
      return jsi::ArrayBuffer(
          rt, std::make_shared<Borrowed>((uint8_t *)c.ptr, c.len));
    default:
      return jsi::Value::undefined();
  }
}

std::string readFile(const char *path) {
  std::ifstream f(path, std::ios::binary);
  std::stringstream s;
  s << f.rdbuf();
  return s.str();
}

/// This executable's path.
std::filesystem::path selfPath(const char *argv0) {
#if defined(__APPLE__)
  char buf[4096];
  uint32_t size = sizeof buf;
  if (_NSGetExecutablePath(buf, &size) == 0) return std::filesystem::canonical(buf);
#elif defined(_WIN32)
  wchar_t buf[4096];
  if (GetModuleFileNameW(nullptr, buf, 4096) > 0) return std::filesystem::path(buf);
#else
  std::error_code e;
  auto p = std::filesystem::canonical("/proc/self/exe", e);
  if (!e) return p;
#endif
  return std::filesystem::absolute(argv0);
}

/// A package's directory: where game.hbc is, beside the executable or in
/// a macOS bundle's Resources.
std::filesystem::path packageDir(const char *argv0) {
  auto dir = selfPath(argv0).parent_path();
  for (auto d : {dir, dir.parent_path() / "Resources"})
    if (std::filesystem::exists(d / "game.hbc")) return d;
  return {};
}

}  // namespace

int main(int argc, char **argv) {
  // The bundle and its environment: the arguments, or the package's.
  std::string bundle;
  std::vector<std::string> pairs;
  if (argc >= 2) {
    bundle = argv[1];
    for (int i = 2; i < argc; i++) pairs.push_back(argv[i]);
  } else {
    auto dir = packageDir(argv[0]);
    if (dir.empty()) {
      std::fprintf(stderr, "usage: og-hermes <bundle.hbc> [key=value ...] (or beside a package's game.hbc)\n");
      return 2;
    }
    bundle = (dir / "game.hbc").string();
    std::ifstream env(dir / "shell.env");
    for (std::string line; std::getline(env, line);)
      if (!line.empty() && line[0] != '#') pairs.push_back(line);
    pairs.push_back("OG_ROOT=" + dir.string());
    pairs.push_back("OG_BIN=" + selfPath(argv[0]).parent_path().string());
    pairs.push_back("OG_PROJECT=@" + (dir / "project.ogt").string());
  }
  for (const char *key : {"OG_SHELL_CAPTURE", "OG_SIZE"})
    if (const char *v = std::getenv(key)) pairs.push_back(std::string(key) + "=" + v);
  // OG_HERMES_PROFILE=<file>: Hermes's sampling profile of the run, in
  // Chrome's trace format.
  const char *profile = std::getenv("OG_HERMES_PROFILE");
  auto *root = jsi::castInterface<facebook::hermes::IHermesRootAPI>(
      facebook::hermes::makeHermesRootAPI());
  if (profile) root->enableSamplingProfiler(2000);

  auto runtime = facebook::hermes::makeHermesRuntime(
      ::hermes::vm::RuntimeConfig::Builder()
          .withMicrotaskQueue(true)
          .withEnableSampleProfiling(std::getenv("OG_HERMES_PROFILE") != nullptr)
          .build());
  jsi::Runtime &rt = *runtime;
  if (!jsi::castInterface<facebook::hermes::IHermesArrayBufferDetach>(&rt)) {
    std::fprintf(stderr,
                 "og-hermes: this Hermes cannot detach an ArrayBuffer; build it with "
                 "third_party/hermes/detach-array-buffer.patch\n");
    return 1;
  }
  shell.rt = &rt;

  jsi::Object host(rt);
  size_t count = 0;
  const OgSymbol *symbols = og_c_symbols(&count);
  for (size_t i = 0; i < count; i++) {
    const OgSymbol s = symbols[i];
    auto name = jsi::PropNameID::forAscii(rt, s.name);
    host.setProperty(
        rt, name,
        jsi::Function::createFromHostFunction(
            rt, name, (unsigned)s.arity,
            [s](jsi::Runtime &rt, const jsi::Value &, const jsi::Value *args,
                size_t n) -> jsi::Value {
              std::vector<std::string> keep;
              keep.reserve(s.arity);
              std::vector<OgValue> in(s.arity);
              for (size_t a = 0; a < s.arity && a < n; a++)
                in[a] = toC(rt, args[a], keep);
              OgValue out{};
              s.call(in.data(), &out);
              return toJs(rt, out);
            }));
  }
  rt.global().setProperty(rt, "__og_host", host);

  static const auto start = std::chrono::steady_clock::now();
  rt.global().setProperty(
      rt, "__og_now",
      jsi::Function::createFromHostFunction(
          rt, jsi::PropNameID::forAscii(rt, "__og_now"), 0,
          [](jsi::Runtime &, const jsi::Value &, const jsi::Value *, size_t) {
            return jsi::Value(std::chrono::duration<double, std::milli>(
                                  std::chrono::steady_clock::now() - start)
                                  .count());
          }));
  rt.global().setProperty(
      rt, "__og_print",
      jsi::Function::createFromHostFunction(
          rt, jsi::PropNameID::forAscii(rt, "__og_print"), 1,
          [](jsi::Runtime &rt, const jsi::Value &, const jsi::Value *args,
             size_t n) {
            if (n > 0) std::printf("%s\n", args[0].toString(rt).utf8(rt).c_str());
            std::fflush(stdout);
            return jsi::Value::undefined();
          }));
  rt.global().setProperty(
      rt, "__og_window_open",
      jsi::Function::createFromHostFunction(
          rt, jsi::PropNameID::forAscii(rt, "__og_window_open"), 5,
          [](jsi::Runtime &rt, const jsi::Value &, const jsi::Value *args,
             size_t n) -> jsi::Value {
            if (n < 5) return jsi::String::createFromAscii(rt, "{\"ok\":false,\"reason\":\"arguments\"}");
            std::string title = args[3].toString(rt).utf8(rt);
            const char *answer = og_c_window_open(
                (uint32_t)args[0].asNumber(), (uint32_t)args[1].asNumber(),
                (uint32_t)args[2].asNumber(), title.c_str(), args[4].getBool());
            return jsi::String::createFromUtf8(rt, answer);
          }));
  rt.global().setProperty(
      rt, "__og_window_fullscreen",
      jsi::Function::createFromHostFunction(
          rt, jsi::PropNameID::forAscii(rt, "__og_window_fullscreen"), 1,
          [](jsi::Runtime &, const jsi::Value &, const jsi::Value *args, size_t n) {
            if (n > 0) og_c_window_fullscreen(args[0].getBool());
            return jsi::Value::undefined();
          }));
  rt.global().setProperty(
      rt, "__og_audio",
      jsi::Function::createFromHostFunction(
          rt, jsi::PropNameID::forAscii(rt, "__og_audio"), 0,
          [](jsi::Runtime &rt, const jsi::Value &, const jsi::Value *, size_t) {
            return jsi::String::createFromUtf8(rt, og_c_window_audio());
          }));
  // Platform services: `__og_data(game[, document])` loads (null for none) or
  // saves the save document; `__og_provider(path)` opens a store's provider
  // library; `__og_provider_call(method, json)` calls it (null for none).
  auto nullable = [](jsi::Runtime &rt, const char *s) -> jsi::Value {
    return s ? jsi::Value(jsi::String::createFromUtf8(rt, s)) : jsi::Value::null();
  };
  rt.global().setProperty(
      rt, "__og_data",
      jsi::Function::createFromHostFunction(
          rt, jsi::PropNameID::forAscii(rt, "__og_data"), 2,
          [nullable](jsi::Runtime &rt, const jsi::Value &, const jsi::Value *args,
                     size_t n) -> jsi::Value {
            if (n < 1) return jsi::Value::null();
            std::string game = args[0].toString(rt).utf8(rt);
            if (n < 2) return nullable(rt, og_c_data_load(game.c_str()));
            std::string doc = args[1].toString(rt).utf8(rt);
            return jsi::Value(og_c_data_save(game.c_str(), doc.c_str()));
          }));
  rt.global().setProperty(
      rt, "__og_provider",
      jsi::Function::createFromHostFunction(
          rt, jsi::PropNameID::forAscii(rt, "__og_provider"), 1,
          [](jsi::Runtime &rt, const jsi::Value &, const jsi::Value *args,
             size_t n) -> jsi::Value {
            std::string path = n > 0 ? args[0].toString(rt).utf8(rt) : "";
            return jsi::String::createFromUtf8(rt, og_c_provider_open(path.c_str()));
          }));
  rt.global().setProperty(
      rt, "__og_provider_call",
      jsi::Function::createFromHostFunction(
          rt, jsi::PropNameID::forAscii(rt, "__og_provider_call"), 2,
          [nullable](jsi::Runtime &rt, const jsi::Value &, const jsi::Value *args,
                     size_t n) -> jsi::Value {
            if (n < 2) return jsi::Value::null();
            std::string method = args[0].toString(rt).utf8(rt);
            std::string json = args[1].toString(rt).utf8(rt);
            return nullable(rt, og_c_provider_call(method.c_str(), json.c_str()));
          }));
  jsi::Object env(rt);
  for (const auto &pair : pairs) {
    auto eq = pair.find('=');
    if (eq == std::string::npos) continue;
    // `key=@file` passes the file's text.
    std::string value = pair[eq + 1] == '@' ? readFile(pair.c_str() + eq + 2) : pair.substr(eq + 1);
    env.setProperty(rt, pair.substr(0, eq).c_str(), jsi::String::createFromUtf8(rt, value));
  }
  // The player's locale, as the OS names it, unless the package gave one.
  if (!env.hasProperty(rt, "OG_LOCALE"))
    if (const char *locale = og_c_locale())
      env.setProperty(rt, "OG_LOCALE", jsi::String::createFromUtf8(rt, locale));
  rt.global().setProperty(rt, "__og_env", env);

  std::string code = readFile(bundle.c_str());
  int status = 0;
  try {
    rt.evaluateJavaScript(std::make_shared<jsi::StringBuffer>(code), bundle);
    rt.drainMicrotasks();
    // The display loop, when the game opened the window.
    jsi::Value frame = rt.global().getProperty(rt, "__og_frame");
    if (frame.isObject() && frame.getObject(rt).isFunction(rt)) {
      jsi::Function step = frame.getObject(rt).getFunction(rt);
      const char *limitText = std::getenv("OG_SHELL_FRAMES");
      long limit = limitText ? std::atol(limitText) : 0;
      // Each display frame at the shell's clock (ms since `origin`, never 0); the frame answers when
      // the next one to draw is due, and the shell waits until then (winit's ControlFlow::WaitUntil).
      auto origin = std::chrono::steady_clock::now() - std::chrono::milliseconds(1);
      auto at = [&] { return std::chrono::duration<double, std::milli>(std::chrono::steady_clock::now() - origin).count(); };
      for (long n = 0; og_c_window_pump() && (limit == 0 || n < limit); n++) {
        double next = step.call(rt, at()).asNumber();
        rt.drainMicrotasks();
        double wait = next - at();
        if (wait > 0) std::this_thread::sleep_for(std::chrono::duration<double, std::milli>(wait));
      }
      if (const char *capture = std::getenv("OG_SHELL_CAPTURE"))
        if (!og_c_window_capture(capture))
          std::fprintf(stderr, "og-hermes: no frame to capture to %s\n", capture);
      jsi::Value report = rt.global().getProperty(rt, "__og_report");
      if (report.isObject() && report.getObject(rt).isFunction(rt))
        std::printf("%s\n", report.getObject(rt).getFunction(rt).call(rt).toString(rt).utf8(rt).c_str());
      // The engine, then the window its renderer presents to.
      jsi::Value close = rt.global().getProperty(rt, "__og_close");
      if (close.isObject() && close.getObject(rt).isFunction(rt))
        close.getObject(rt).getFunction(rt).call(rt);
      og_c_window_close();
    }
  } catch (const jsi::JSIException &e) {
    std::fprintf(stderr, "og-hermes: %s\n", e.what());
    status = 1;
  }
  if (profile) {
    root->dumpSampledTraceToFile(profile);
    root->disableSamplingProfiler();
  }
  // The JavaScript values the shell holds go before the runtime does.
  shell.views.clear();
  shell.dispatcher.reset();
  std::fflush(stdout);
  return status;
}
