use topcoat::{
    Result,
    icon::{icon, iconify},
    view::{View, attributes, component, view},
};

use crate::components::button::{ButtonSize, ButtonVariant, button};

iconify::include!("feather");

#[component]
pub async fn theme_button() -> Result<impl View> {
    Ok(view! {
        button(
            variant: ButtonVariant::Ghost,
            size: ButtonSize::Icon,
            attrs: attributes! {
                aria-label="Toggle theme"
                onclick="
                        const dark = document.documentElement.classList.toggle('dark');
                        localStorage.setItem('theme', dark ? 'dark' : 'light');
                      "
            },
            // Visible in light mode: show a moon ("switch to dark")
            <span class="dark:hidden">icon(data: feather::MOON, label: "Dark")</span>
            // Visible in dark mode: show a sun ("switch to light")
            <span class="hidden dark:inline">
                icon(data: feather::SUN, label: "Light")
            </span>
        )
    })
}
