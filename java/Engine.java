public class Engine {

	public static native void hello();

	static {
		System.loadLibrary("flexbox");
	}

    public static void main(String[] args) {
        View view = new View();
        System.out.println(view.getNode());
        hello();
    }

}
