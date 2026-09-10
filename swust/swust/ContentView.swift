import SwiftUI

struct ContentView: View {
    @State private var player: Player? = nil
    @State private var isPlaying: Bool = false

    var body: some View {
        VStack {
            if let player = player {
                MetalCanvas(player: player)
                    .aspectRatio(16.0 / 9.0, contentMode: .fit)
                    .frame(maxWidth: .infinity)
            }

            Button(isPlaying ? "pause" : "play") {
                guard let player = player else { return }
                if isPlaying {
                    player.pause()
                } else {
                    player.play()
                }
                isPlaying.toggle()
            }
        }
        .task {
            do {
                player = try Player(url: "http://localhost:8080")
            } catch {
                print("Failed to create player: \(error)")
            }
        }
    }
}

#Preview {
    ContentView()
}
