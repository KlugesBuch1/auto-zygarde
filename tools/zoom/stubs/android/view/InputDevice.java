package android.view;

public class InputDevice {
    public static int SOURCE_TOUCHSCREEN = 0x00001002;

    public static int[] getDeviceIds() {
        return new int[0];
    }

    public static InputDevice getDevice(int id) {
        return null;
    }

    public boolean supportsSource(int source) {
        return false;
    }
}
