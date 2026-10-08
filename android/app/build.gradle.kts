plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
}

val botAssets = layout.buildDirectory.dir("generated/bot-assets")

val botBinary = if (project.hasProperty("botBinary")) {
    file(project.property("botBinary") as String)
} else {
    rootProject.file("../target/aarch64-linux-android/release/auto-zygarde")
}

val stageBotAssets = tasks.register<Copy>("stageBotAssets") {
    into(botAssets)
    doFirst {
        if (!botBinary.exists()) {
            throw GradleException("Android-Binary fehlt: ${botBinary.absolutePath}")
        }
    }
    from(rootProject.file("../assets/templates")) {
        into("templates")
        exclude(".gitkeep")
    }
    from(rootProject.file("../assets/gpx/nyc")) {
        include("*.gpx")
        into("gpx")
    }
    from(botBinary) {
        into("bin")
    }
}

android {
    namespace = "com.autozygarde"
    compileSdk = 34

    defaultConfig {
        applicationId = "com.autozygarde"
        minSdk = 26
        targetSdk = 34
        versionCode = 1
        versionName = "0.1.0"
    }

    buildTypes {
        release {
            isMinifyEnabled = false
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    kotlinOptions {
        jvmTarget = "17"
    }

    sourceSets {
        getByName("main") {
            assets.srcDir(botAssets)
        }
    }
}

tasks.named("preBuild").configure {
    dependsOn(stageBotAssets)
}
