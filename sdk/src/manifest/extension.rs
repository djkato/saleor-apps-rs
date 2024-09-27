use super::{AppExtension, AppExtensionMount, AppExtensionTarget, AppPermission};

pub struct AppExtensionBuilder {
    pub extension: AppExtension,
}

impl AppExtensionBuilder {
    pub fn new() -> Self {
        Self {
            extension: AppExtension::default(),
        }
    }
    /** Name which will be displayed in the dashboard */
    pub fn set_label(mut self, label: &str) -> Self {
        label.clone_into(&mut self.extension.label);
        self
    }

    /** the place where the extension will be mounted */
    pub fn set_mount(mut self, mount: AppExtensionMount) -> Self {
        self.extension.mount = mount;
        self
    }

    /** Method of presenting the interface
      `POPUP` will present the interface in a modal overlay
      `APP_PAGE` will navigate to the application page
      @default `POPUP`
    */
    pub fn set_target(mut self, target: AppExtensionTarget) -> Self {
        self.extension.target = target;
        self
    }

    pub fn add_permission(mut self, permission: AppPermission) -> Self {
        self.extension.permissions.push(permission);
        self
    }

    pub fn add_permissions(mut self, mut permission: Vec<AppPermission>) -> Self {
        self.extension.permissions.append(&mut permission);
        self
    }

    /** URL of the view to display,
     you can skip the domain and protocol when target is set to `APP_PAGE`, or when your manifest defines an `appUrl`.

     When target is set to `POPUP`, the url will be used to render an `<iframe>`.
    */
    pub fn set_url(mut self, url: &str) -> Self {
        url.clone_into(&mut self.extension.url);
        self
    }

    pub fn build(self) -> AppExtension {
        self.extension
    }
}
