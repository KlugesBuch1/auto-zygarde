package android.view;

public class MotionEvent extends InputEvent {
    public static int ACTION_DOWN = 0;
    public static int ACTION_UP = 1;
    public static int ACTION_MOVE = 2;
    public static int ACTION_POINTER_DOWN = 5;
    public static int ACTION_POINTER_UP = 6;
    public static int ACTION_POINTER_INDEX_SHIFT = 8;
    public static int TOOL_TYPE_FINGER = 1;

    public static class PointerProperties {
        public int id;
        public int toolType;
    }

    public static class PointerCoords {
        public float x;
        public float y;
        public float pressure;
        public float size;
    }

    public static MotionEvent obtain(
            long downTime,
            long eventTime,
            int action,
            int pointerCount,
            PointerProperties[] pointerProperties,
            PointerCoords[] pointerCoords,
            int metaState,
            int buttonState,
            float xPrecision,
            float yPrecision,
            int deviceId,
            int edgeFlags,
            int source,
            int flags) {
        return null;
    }

    public void recycle() {
    }
}
