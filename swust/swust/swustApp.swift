//
//  swustApp.swift
//  swust
//
//  Created by Yohai Wiener on 08/01/2026.
//

import SwiftUI

@main
struct swustApp: App {
    init() {
        Logger.setup()
    }

    var body: some Scene {
        WindowGroup {
            ContentView()
        }
    }
}
