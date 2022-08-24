//TODO: Check if macro can become private.
macro_rules! menu_list {
    ($($internal:tt)*) => {{
        let mut menu = menu_list_complex! {$($internal)*};

        menu
    }}
}

macro_rules! menu_list_complex {
    ($name:literal: {$($internal:tt)*}) => {{
        let mut internal_vector = menu_list_whole! {$($internal)*};
        internal_vector.reverse();

        MenuList::generate_non_final($name, internal_vector)
    }};
}

macro_rules! menu_list_whole {
    () => {{
        Vec::new() as Children
    }};

    ($name:literal) => {{
        let single_list: MenuList = MenuList::generate_final($name);
        let mut single_vector: Children = Vec::new();

        single_vector.push(Rc::new(Box::new(single_list)));

        single_vector
    }};

    ($name:literal: {$($internal:tt)*}) => {{
        let internal_list: MenuList = menu_list_complex! {$name: {$($internal)*}};
        let mut single_vec: Children = Vec::new();

        single_vec.push(Rc::new(Box::new(internal_list)));

        single_vec
    }};

    ($name:literal, $($external:tt)*) => {{
        let single_list: MenuList = MenuList::generate_final($name);
        let mut external_vec: Children = menu_list_whole! {$($external)*};

        external_vec.push(Rc::new(Box::new(single_list)));

        external_vec
    }};

    ($name:literal: {$($internal:tt)*}, $($external:tt)*) => {{
        let internal_list: MenuList = menu_list_complex! {$name: {$($internal)*}};
        let mut external_vec: Children = menu_list_whole! {$($external)*};

        external_vec.push(Rc::new(Box::new(internal_list)));

        external_vec
    }};
}
