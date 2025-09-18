#[macro_export]
macro_rules! use_state {
    ($cx:expr, $value:expr) => {
        $cx.use_state($value)
    };
}

#[macro_export]
macro_rules! use_effect {
    ($cx:expr, $callback:expr $(,)?) => {
      $cx.use_effect($callback, 0)
    };
    ($cx:expr, $callback:expr, $( $dep:expr ),+ $(,)?) => {
      $cx.use_effect($callback, &[$( $dep ),+])
    };
}

#[macro_export]
macro_rules! use_memo {
  ($cx:expr, $callback:expr $(,)?) => {
    {
      let (state, set_state) = $cx.use_state_factory($callback);
      state
    }
  };
  ($cx:expr, $callback:expr, $( $dep:expr ),+ $(,)?) => {
    {
      let (state, set_state) = $cx.use_state_factory($callback);

      $cx.use_effect(move || {
        set_state($callback());
        || {}
      }, &[$( $dep ),+]);

      state
    }
  };
}

#[macro_export]
macro_rules! use_async_memo {
  ($cx:expr, $callback:expr $(,)?) => {
    {
      let (state, set_state) = $cx.use_state_factory(None);

      $cx.use_effect(|| {
        let set_state = set_state.clone();
        let task = tokio::spawn(async move {
          set_state(Some($callback().await));
        });
        move || task.abort()
      }, 0);

      state
    }
  };
  ($cx:expr, $callback:expr, $( $dep:expr ),+ $(,)?) => {
    {
      let (state, set_state) = $cx.use_state(None);

      $cx.use_effect(|| {
        let set_state = set_state.clone();
        let task = tokio::spawn(async move {
          set_state(Some($callback().await));
        });
        move || task.abort()
      }, &[$( $dep ),+]);

      state
    }
  };
}
