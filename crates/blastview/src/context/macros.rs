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
      let (is_loading, set_is_loading) = $cx.use_state(false);

      $cx.use_effect(|| {
        let task = tokio::spawn(async move {
          set_is_loading(true);
          let result = $callback().await;
          tokio::task::yield_now().await;
          set_state(Some(result));
          set_is_loading(false);
        });
        move || task.abort()
      }, 0);

      (is_loading, state)
    }
  };
  ($cx:expr, $callback:expr, $( $dep:expr ),+ $(,)?) => {
    {
      let (state, set_state) = $cx.use_state(None);
      let (is_loading, set_is_loading) = $cx.use_state(true);

      $cx.use_effect(|| {
        let task = tokio::spawn(async move {
          set_is_loading(true);
          let result = $callback().await;
          tokio::task::yield_now().await;
          set_state(Some(result));
          set_is_loading(false);
        });
        move || task.abort()
      }, &[$( $dep ),+]);

      (is_loading, state)
    }
  };
}
