package silo;

import java.io.*;

/** Minimal JSON-lines writer (no dependency). Values: String, Number, Boolean, int[], null. */
public final class Jsonl implements Closeable {
    private final PrintWriter w;
    public Jsonl(File f) throws IOException { w = new PrintWriter(new BufferedWriter(new FileWriter(f)), false); }
    public static String esc(String s) {
        StringBuilder sb = new StringBuilder("\"");
        for (char c : s.toCharArray()) {
            switch (c) {
                case '"': sb.append("\\\""); break; case '\\': sb.append("\\\\"); break; case '\n': sb.append("\\n"); break;
                case '\r': break; case '\t': sb.append("\\t"); break;
                default: if (c < 0x20) sb.append(String.format("\\u%04x", (int) c)); else sb.append(c);
            }
        }
        return sb.append('"').toString();
    }
    public synchronized void write(Object... kv) {
        StringBuilder sb = new StringBuilder("{");
        for (int i = 0; i < kv.length; i += 2) {
            if (i > 0) sb.append(',');
            sb.append(esc((String) kv[i])).append(':');
            Object v = kv[i + 1];
            if (v == null) sb.append("null");
            else if (v instanceof String s) sb.append(esc(s));
            else if (v instanceof int[] a) { sb.append('['); for (int j = 0; j < a.length; j++) { if (j > 0) sb.append(','); sb.append(a[j]); } sb.append(']'); }
            else sb.append(v);
        }
        w.println(sb.append('}')); 
    }
    public synchronized void flush() { w.flush(); }
    @Override public void close() { w.close(); }
}
