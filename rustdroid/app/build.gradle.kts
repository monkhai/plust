plugins {
    alias(libs.plugins.android.application)
    alias(libs.plugins.kotlin.android)
    alias(libs.plugins.kotlin.compose)
    alias(libs.plugins.kotlin.serialization)
}

val rustProjectDir = rootProject.projectDir.parentFile.resolve("plust-core")
val jniLibsDir = layout.buildDirectory.dir("rustJniLibs")

tasks.register<Exec>("buildRust") {
    workingDir = rustProjectDir
    commandLine("cargo", "ndk", "-t", "arm64-v8a", "-o", jniLibsDir.get().asFile.absolutePath, "build", "--release")
}

tasks.named("preBuild") {
    dependsOn("buildRust")
}

android {
    namespace = "com.example.rustdroid"
    compileSdk {
        version = release(36)
    }

    sourceSets {
        getByName("main") {
            jniLibs.srcDir(jniLibsDir)
        }
    }

    defaultConfig {
        applicationId = "com.example.rustdroid"
        minSdk = 24
        targetSdk = 36
        versionCode = 1
        versionName = "1.0"

        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
    }

    buildTypes {
        release {
            isMinifyEnabled = false
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro"
            )
        }
    }
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_11
        targetCompatibility = JavaVersion.VERSION_11
    }
    kotlinOptions {
        jvmTarget = "11"
    }
    buildFeatures {
        compose = true
    }
}

dependencies {
    implementation("net.java.dev.jna:jna:5.14.0@aar")
    implementation(libs.kotlinx.serialization.json)
    implementation(libs.androidx.core.ktx)
    implementation(libs.androidx.lifecycle.runtime.ktx)
    implementation(libs.androidx.activity.compose)
    implementation(platform(libs.androidx.compose.bom))
    implementation(libs.androidx.compose.ui)
    implementation(libs.androidx.compose.ui.graphics)
    implementation(libs.androidx.compose.ui.tooling.preview)
    implementation(libs.androidx.compose.material3)
    testImplementation(libs.junit)
    androidTestImplementation(libs.androidx.junit)
    androidTestImplementation(libs.androidx.espresso.core)
    androidTestImplementation(platform(libs.androidx.compose.bom))
    androidTestImplementation(libs.androidx.compose.ui.test.junit4)
    debugImplementation(libs.androidx.compose.ui.tooling)
    debugImplementation(libs.androidx.compose.ui.test.manifest)
}
