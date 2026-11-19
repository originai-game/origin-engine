// The prototype native shell: Hermes running a game's bytecode on the C ABI
// host (og-host-c). It registers every entry of og_c_symbols() as a function
// of `globalThis.__og_host`, turns OgValues into JavaScript values and back,
// and gives the game's dispatcher to the engine. No window and no renderer:
// it grounds the shell's design (README.md) and runs the game-frame
// benchmark (packages/engine/bench/game-frame/hermes.ts) under Hermes.
//
//   og-hermes <bundle.hbc> [key=value ...]
//
// Each key=value becomes `globalThis.__og_env[key]` (key=@file: the file's text).

#include <hermes/hermes.h>
#include <jsi/jsi.h>

#include <chrono>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <fstream>
#include <memory>
#include <sstream>
#include <string>
#include <vector>

#include "og_host.h"

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
  // A kept typed array reads as empty instead of reaching moved memory.
  auto detach = rt.global()
                    .getPropertyAsObject(rt, "HermesInternal")
                    .getPropertyAsFunction(rt, "detachArrayBuffer");
  for (auto &v : shell.views) detach.call(rt, v);
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

}  // namespace

int main(int argc, char **argv) {
  if (argc < 2) {
    std::fprintf(stderr, "usage: og-hermes <bundle.hbc> [key=value ...]\n");
    return 2;
  }
  // OG_HERMES_PROFILE=<file>: Hermes's sampling profile of the run, in
  // Chrome's trace format.
  const char *profile = std::getenv("OG_HERMES_PROFILE");
  auto *root = jsi::castInterface<facebook::hermes::IHermesRootAPI>(
      facebook::hermes::makeHermesRootAPI());
  if (profile) root->enableSamplingProfiler(2000);

  auto runtime = facebook::hermes::makeHermesRuntime(
      ::hermes::vm::RuntimeConfig::Builder()
          .withEnableHermesInternalTestMethods(true)
          .withMicrotaskQueue(true)
          .withEnableSampleProfiling(std::getenv("OG_HERMES_PROFILE") != nullptr)
          .build());
  jsi::Runtime &rt = *runtime;
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
  jsi::Object env(rt);
  for (int i = 2; i < argc; i++) {
    const char *eq = std::strchr(argv[i], '=');
    if (!eq) continue;
    // `key=@file` passes the file's text.
    std::string value = eq[1] == '@' ? readFile(eq + 2) : std::string(eq + 1);
    env.setProperty(rt, std::string(argv[i], eq - argv[i]).c_str(),
                    jsi::String::createFromUtf8(rt, value));
  }
  rt.global().setProperty(rt, "__og_env", env);

  std::string code = readFile(argv[1]);
  int status = 0;
  try {
    rt.evaluateJavaScript(std::make_shared<jsi::StringBuffer>(code), argv[1]);
    rt.drainMicrotasks();
  } catch (const jsi::JSIException &e) {
    std::fprintf(stderr, "og-hermes: %s\n", e.what());
    status = 1;
  }
  if (profile) {
    root->dumpSampledTraceToFile(profile);
    root->disableSamplingProfiler();
  }
  return status;
}
