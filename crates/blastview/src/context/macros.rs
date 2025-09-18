#[macro_export]
macro_rules! use_state {
    ($cx:expr, $value:expr) => {
        $cx.use_state($value)
    };
}

#[macro_export]
macro_rules! use_effect {
    ($cx:expr, $callback:expr $(,)?) => {
      $cx.use_effect($callback, &[] as &[u8])
    };
    ($cx:expr, $callback:expr, $( $dep:expr ),+ $(,)?) => {
      $cx.use_effect($callback, &[$( $dep ),+])
    };
}

#[macro_export]
macro_rules! use_memo {
  ($cx:expr, $callback:expr $(,)?) => {
    {
      let (state, set_state) = $cx.use_state($callback());
      state
    }
  };
  ($cx:expr, $callback:expr, $( $dep:expr ),+ $(,)?) => {
    {
      let (state, set_state) = $cx.use_state($callback());

      $cx.use_effect(move || {
        set_state($callback());
      }, &[$( $dep ),+])

      state
    }
  };
}
