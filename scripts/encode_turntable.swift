// Assemble actual native PNG captures; no synthesized frames or geometry.
import Foundation
import ImageIO
import UniformTypeIdentifiers

guard CommandLine.arguments.count == 3 else {
    fatalError("Usage: swift scripts/encode_turntable.swift frames-directory output.gif")
}
let directory = URL(fileURLWithPath: CommandLine.arguments[1], isDirectory: true)
let output = URL(fileURLWithPath: CommandLine.arguments[2])
let frames = try FileManager.default.contentsOfDirectory(at: directory, includingPropertiesForKeys: nil)
    .filter { $0.lastPathComponent.hasPrefix("frame-") && $0.pathExtension == "png" }
    .sorted { $0.lastPathComponent < $1.lastPathComponent }
guard !frames.isEmpty, let destination = CGImageDestinationCreateWithURL(output as CFURL,
        UTType.gif.identifier as CFString, frames.count, nil) else { fatalError("No frames/output") }
CGImageDestinationSetProperties(destination,
    [kCGImagePropertyGIFDictionary: [kCGImagePropertyGIFLoopCount: 0]] as CFDictionary)
for frame in frames {
    guard let source = CGImageSourceCreateWithURL(frame as CFURL, nil),
          let image = CGImageSourceCreateImageAtIndex(source, 0, nil) else { fatalError("Invalid PNG") }
    CGImageDestinationAddImage(destination, image,
        [kCGImagePropertyGIFDictionary: [kCGImagePropertyGIFDelayTime: 0.16,
                                       kCGImagePropertyGIFUnclampedDelayTime: 0.16]] as CFDictionary)
}
guard CGImageDestinationFinalize(destination) else { fatalError("GIF encoding failed") }
guard let decoded = CGImageSourceCreateWithURL(output as CFURL, nil),
      CGImageSourceGetCount(decoded) == frames.count else { fatalError("Incomplete GIF") }
print("Encoded \(frames.count) native frames: \(output.path)")
