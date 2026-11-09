// Dump the macOS accessibility (AX/AccessKit) tree of a GPUI process as JSON.
// This is what VoiceOver reads: AXRole/AXTitle/AXDescription/AXValue/AXEnabled.
// AccessKit only builds the tree after an assistive client queries the app, so
// `tree` touches AXWindows once to activate it, then polls until two
// consecutive walks agree on a node count > 8.
//
//   ax_dump tree  <pid> [--settle-ms N]
//   ax_dump focus <pid> <path>   # path like "0,5,2" (indices from the window)
//   ax_dump press <pid> <path>
//
// Compile: swiftc -O ax_dump.swift -o ax_dump
import ApplicationServices
import Foundation

func attr(_ el: AXUIElement, _ name: String) -> AnyObject? {
    var value: AnyObject?
    let err = AXUIElementCopyAttributeValue(el, name as CFString, &value)
    return err == .success ? value : nil
}

func childrenOf(_ el: AXUIElement) -> [AXUIElement] {
    (attr(el, kAXChildrenAttribute) as? [AXUIElement]) ?? []
}

func appElement(_ pid: Int32) -> AXUIElement {
    AXUIElementCreateApplication(pid_t(pid))
}

func firstWindow(_ app: AXUIElement) -> AXUIElement? {
    (attr(app, kAXWindowsAttribute) as? [AXUIElement])?.first
}

/// Resolve a "0,5,2" or "[0,5,2]" path to the element `path` indices below the
/// window's own children (the window itself is path "").
func nodeAtPath(_ window: AXUIElement, _ path: String) -> AXUIElement? {
    let cleaned = path.trimmingCharacters(in: .whitespaces)
        .replacingOccurrences(of: "[", with: "")
        .replacingOccurrences(of: "]", with: "")
    guard !cleaned.isEmpty else { return window }
    var el = window
    for part in cleaned.split(separator: ",") {
        guard let idx = Int(part.trimmingCharacters(in: .whitespaces)),
              idx >= 0, idx < childrenOf(el).count else { return nil }
        el = childrenOf(el)[idx]
    }
    return el
}

/// JSON scalar for an AX attribute: strings stay strings, booleans stay booleans,
/// other numbers stay numbers, everything else becomes its description.
func scalar(_ v: AnyObject?) -> Any {
    guard let v else { return NSNull() }
    if let s = v as? String { return s }
    if let n = v as? NSNumber {
        if CFGetTypeID(n) == CFBooleanGetTypeID() { return n.boolValue }
        return n.doubleValue
    }
    return String(describing: v)
}

func axPoint(_ el: AXUIElement, _ name: String) -> CGPoint? {
    guard let raw = attr(el, name), CFGetTypeID(raw) == AXValueGetTypeID() else { return nil }
    var p = CGPoint.zero
    guard AXValueGetValue(raw as! AXValue, .cgPoint, &p) else { return nil }
    return p
}

func axSize(_ el: AXUIElement, _ name: String) -> CGSize? {
    guard let raw = attr(el, name), CFGetTypeID(raw) == AXValueGetTypeID() else { return nil }
    var s = CGSize.zero
    guard AXValueGetValue(raw as! AXValue, .cgSize, &s) else { return nil }
    return s
}

func frameDict(_ el: AXUIElement) -> [String: Double]? {
    guard let p = axPoint(el, kAXPositionAttribute),
          let s = axSize(el, kAXSizeAttribute) else { return nil }
    return ["x": Double(p.x), "y": Double(p.y), "w": Double(s.width), "h": Double(s.height)]
}

/// The title/value text of the node an AXTitleUIElement points at, if any.
func titleElementText(_ el: AXUIElement) -> String? {
    guard let target = attr(el, kAXTitleUIElementAttribute) else { return nil }
    let t = attr(target as! AXUIElement, kAXTitleAttribute)
    if let s = t as? String, !s.isEmpty { return s }
    let v = attr(target as! AXUIElement, kAXValueAttribute)
    if let s = v as? String, !s.isEmpty { return s }
    return nil
}

func actionsOf(_ el: AXUIElement) -> [String] {
    var names: CFArray?
    guard AXUIElementCopyActionNames(el, &names) == .success, let names else { return [] }
    return (names as? [String]) ?? []
}

/// Serialize one node (and, recursively, its children). `path` is the index
/// path from the window's children; the window's direct children are [i].
func nodeDict(_ el: AXUIElement, _ path: [Int]) -> [String: Any] {
    var out: [String: Any] = [:]
    out["role"] = scalar(attr(el, kAXRoleAttribute))
    out["subrole"] = scalar(attr(el, kAXSubroleAttribute))
    out["roleDescription"] = scalar(attr(el, kAXRoleDescriptionAttribute))
    out["title"] = scalar(attr(el, kAXTitleAttribute))
    out["description"] = scalar(attr(el, kAXDescriptionAttribute))
    out["value"] = scalar(attr(el, kAXValueAttribute))
    out["help"] = scalar(attr(el, kAXHelpAttribute))
    out["placeholder"] = scalar(attr(el, kAXPlaceholderValueAttribute))
    out["enabled"] = scalar(attr(el, kAXEnabledAttribute))
    out["focused"] = scalar(attr(el, kAXFocusedAttribute))
    out["minValue"] = scalar(attr(el, kAXMinValueAttribute))
    out["maxValue"] = scalar(attr(el, kAXMaxValueAttribute))
    if let f = frameDict(el) { out["frame"] = f }
    if let te = titleElementText(el) { out["titleElement"] = te }
    out["actions"] = actionsOf(el)
    out["path"] = path
    let kids = childrenOf(el)
    out["children"] = kids.enumerated().map { nodeDict($0.element, path + [$0.offset]) }
    return out
}

func countNodes(_ nodes: [[String: Any]]) -> Int {
    var n = 0
    for node in nodes {
        n += 1
        if let kids = node["children"] as? [[String: Any]] { n += countNodes(kids) }
    }
    return n
}

func walkWindow(_ window: AXUIElement) -> [[String: Any]] {
    childrenOf(window).enumerated().map { nodeDict($0.element, [$0.offset]) }
}

func emitJSON(_ obj: Any) {
    if let data = try? JSONSerialization.data(withJSONObject: obj, options: [.prettyPrinted, .sortedKeys]),
       let s = String(data: data, encoding: .utf8) {
        print(s)
    }
}

func trustedOrExit() {
    guard AXIsProcessTrusted() else {
        FileHandle.standardError.write("ax_dump: Accessibility permission not granted\n".data(using: .utf8)!)
        exit(3)
    }
}

let args = Array(CommandLine.arguments.dropFirst())
guard let command = args.first else {
    FileHandle.standardError.write("usage: ax_dump tree|focus|press <pid> [path] [--settle-ms N]\n".data(using: .utf8)!)
    exit(64)
}

switch command {
case "tree":
    guard args.count >= 2, let pid = Int32(args[1]) else { exit(64) }
    var settleMs = 5000
    if let i = args.firstIndex(of: "--settle-ms"), i + 1 < args.count {
        settleMs = Int(args[i + 1]) ?? settleMs
    }
    trustedOrExit()
    let app = appElement(pid)
    guard let window = firstWindow(app) else {
        FileHandle.standardError.write("ax_dump: no window for pid \(pid)\n".data(using: .utf8)!)
        exit(4)
    }
    // First touch activates AccessKit; then poll until two walks agree.
    _ = childrenOf(window)
    var lastCount = -1
    var lastNodes: [[String: Any]] = []
    let deadline = Date().addingTimeInterval(Double(settleMs) / 1000.0)
    while Date() < deadline {
        let nodes = walkWindow(window)
        let count = countNodes(nodes)
        if count == lastCount && count > 8 {
            emitJSON(["trusted": true, "nodes": nodes, "node_count": count])
            exit(0)
        }
        lastCount = count
        lastNodes = nodes
        Thread.sleep(forTimeInterval: 0.25)
    }
    // Timed out: report the last walk rather than nothing (caller judges count).
    emitJSON(["trusted": true, "nodes": lastNodes, "node_count": lastCount, "settled": false])
    exit(0)

case "focus", "press":
    guard args.count >= 3, let pid = Int32(args[1]) else { exit(64) }
    let path = args[2]
    trustedOrExit()
    let app = appElement(pid)
    guard let window = firstWindow(app), let target = nodeAtPath(window, path) else {
        FileHandle.standardError.write("ax_dump: cannot resolve path \(path)\n".data(using: .utf8)!)
        exit(4)
    }
    if command == "focus" {
        AXUIElementSetAttributeValue(target, kAXFocusedAttribute as CFString, kCFBooleanTrue)
    } else {
        AXUIElementPerformAction(target, kAXPressAction as CFString)
    }
    Thread.sleep(forTimeInterval: 0.3)
    if command == "focus" {
        if let focused = attr(app, kAXFocusedUIElementAttribute) {
            emitJSON(nodeDict(focused as! AXUIElement, []))
        } else {
            emitJSON(["role": NSNull()])
        }
    } else {
        emitJSON(nodeDict(target, []))
    }

default:
    FileHandle.standardError.write("ax_dump: unknown command \(command)\n".data(using: .utf8)!)
    exit(64)
}
