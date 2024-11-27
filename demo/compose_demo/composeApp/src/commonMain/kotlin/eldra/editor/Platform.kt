package eldra.editor

interface Platform {
    val name: String
}

expect fun getPlatform(): Platform