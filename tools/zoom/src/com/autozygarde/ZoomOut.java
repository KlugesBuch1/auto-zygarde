package com.autozygarde;

import android.hardware.input.InputManager;
import android.os.SystemClock;
import android.view.InputDevice;
import android.view.InputEvent;
import android.view.MotionEvent;

import java.lang.reflect.Method;

public final class ZoomOut {
    private static final int WAIT_FOR_FINISH = 2;

    public static void main(String[] args) throws Exception {
        if (args.length != 9) {
            System.err.println("usage: ZoomOut x1 y1 x2 y2 x3 y3 x4 y4 duration_ms");
            System.exit(2);
        }
        float x1 = Float.parseFloat(args[0]);
        float y1 = Float.parseFloat(args[1]);
        float x2 = Float.parseFloat(args[2]);
        float y2 = Float.parseFloat(args[3]);
        float x3 = Float.parseFloat(args[4]);
        float y3 = Float.parseFloat(args[5]);
        float x4 = Float.parseFloat(args[6]);
        float y4 = Float.parseFloat(args[7]);
        int duration = Integer.parseInt(args[8]);
        if (duration < 1) {
            duration = 1;
        }

        Method getInstance = InputManager.class.getDeclaredMethod("getInstance");
        getInstance.setAccessible(true);
        InputManager manager = (InputManager) getInstance.invoke(null);
        Method inject = InputManager.class.getMethod(
                "injectInputEvent", InputEvent.class, int.class);
        inject.setAccessible(true);

        int deviceId = touchscreenId();
        long down = SystemClock.uptimeMillis();
        int steps = Math.max(8, duration / 16);

        send(manager, inject, deviceId, down, down, MotionEvent.ACTION_DOWN, x1, y1, x2, y2, 1);
        send(manager, inject, deviceId, down, down + 1,
                MotionEvent.ACTION_POINTER_DOWN | (1 << MotionEvent.ACTION_POINTER_INDEX_SHIFT),
                x1, y1, x2, y2, 2);

        for (int i = 1; i <= steps; i++) {
            float t = i / (float) steps;
            long when = down + 1 + (long) duration * i / steps;
            send(manager, inject, deviceId, down, when, MotionEvent.ACTION_MOVE,
                    lerp(x1, x3, t), lerp(y1, y3, t),
                    lerp(x2, x4, t), lerp(y2, y4, t), 2);
            long pause = when - SystemClock.uptimeMillis();
            if (pause > 0) {
                Thread.sleep(pause);
            }
        }

        long up = down + duration + 2;
        send(manager, inject, deviceId, down, up,
                MotionEvent.ACTION_POINTER_UP | (1 << MotionEvent.ACTION_POINTER_INDEX_SHIFT),
                x3, y3, x4, y4, 2);
        send(manager, inject, deviceId, down, up + 1, MotionEvent.ACTION_UP, x3, y3, x4, y4, 1);
    }

    private static int touchscreenId() {
        int[] ids = InputDevice.getDeviceIds();
        for (int i = 0; i < ids.length; i++) {
            InputDevice device = InputDevice.getDevice(ids[i]);
            if (device != null && device.supportsSource(InputDevice.SOURCE_TOUCHSCREEN)) {
                return ids[i];
            }
        }
        return 0;
    }

    private static float lerp(float from, float to, float t) {
        return from + (to - from) * t;
    }

    private static void send(
            InputManager manager,
            Method inject,
            int deviceId,
            long down,
            long when,
            int action,
            float ax,
            float ay,
            float bx,
            float by,
            int count) throws Exception {
        MotionEvent.PointerProperties[] props = new MotionEvent.PointerProperties[count];
        MotionEvent.PointerCoords[] coords = new MotionEvent.PointerCoords[count];
        props[0] = finger(0);
        coords[0] = point(ax, ay);
        if (count > 1) {
            props[1] = finger(1);
            coords[1] = point(bx, by);
        }
        MotionEvent event = MotionEvent.obtain(
                down,
                when,
                action,
                count,
                props,
                coords,
                0,
                0,
                1f,
                1f,
                deviceId,
                0,
                InputDevice.SOURCE_TOUCHSCREEN,
                0);
        Object result = inject.invoke(manager, event, Integer.valueOf(WAIT_FOR_FINISH));
        event.recycle();
        if (result instanceof Boolean && !((Boolean) result).booleanValue()) {
            throw new IllegalStateException("injectInputEvent rejected action " + action);
        }
    }

    private static MotionEvent.PointerProperties finger(int id) {
        MotionEvent.PointerProperties properties = new MotionEvent.PointerProperties();
        properties.id = id;
        properties.toolType = MotionEvent.TOOL_TYPE_FINGER;
        return properties;
    }

    private static MotionEvent.PointerCoords point(float x, float y) {
        MotionEvent.PointerCoords coords = new MotionEvent.PointerCoords();
        coords.x = x;
        coords.y = y;
        coords.pressure = 1f;
        coords.size = 1f;
        return coords;
    }
}
