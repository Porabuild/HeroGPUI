"""InputOTP page: the nine v3 examples it was missing."""
import io

P = 'gallery/src/pages/components.rs'
s = io.open(P, encoding='utf-8', newline='').read()


def rep(old, new):
    global s
    assert s.count(old) == 1, (s.count(old), old[:70])
    s = s.replace(old, new)


rep("""            vec![(
                "Usage",
                col(vec![
                    h::InputOTP::new(self.otp.clone())
                        .on_complete(cx.listener(|this, code: &str, _, cx| {
                            this.otp_done = code.to_owned();
                            cx.notify();
                        }))
                        .into_any_element(),
                    para(
                        &if done.is_empty() {
                            "Enter six digits".to_owned()
                        } else {
                            format!("Complete: {done}")
                        },
                        cx,
                    ),
                ]),
            )],""",
    """            vec![
                (
                    "Usage",
                    col(vec![
                        h::InputOTP::new(self.otp.clone())
                            .on_complete(cx.listener(|this, code: &str, _, cx| {
                                this.otp_done = code.to_owned();
                                cx.notify();
                            }))
                            .into_any_element(),
                        para(
                            &if done.is_empty() {
                                "Enter six digits".to_owned()
                            } else {
                                format!("Complete: {done}")
                            },
                            cx,
                        ),
                    ]),
                ),
                (
                    "Variants",
                    col(vec![
                        h::InputOTP::new(self.demo_otp("otp-primary", 6, cx)).into_any_element(),
                        h::InputOTP::new(self.demo_otp("otp-secondary", 6, cx))
                            .variant(FieldVariant::Secondary)
                            .into_any_element(),
                    ]),
                ),
                (
                    "In Surface",
                    col(vec![h::Surface::new()
                        .padding(px(24.))
                        .child(
                            h::InputOTP::new(self.demo_otp("otp-surface", 6, cx))
                                .variant(FieldVariant::Secondary),
                        )
                        .into_any_element()]),
                ),
                (
                    "Disabled State",
                    col(vec![h::InputOTP::new(self.demo_otp("otp-disabled", 6, cx))
                        .is_disabled(true)
                        .into_any_element()]),
                ),
                (
                    "Four Digits",
                    col(vec![h::InputOTP::new(self.demo_otp("otp-four", 4, cx))
                        .into_any_element()]),
                ),
                (
                    "Controlled",
                    col(vec![
                        h::InputOTP::new(self.demo_otp("otp-controlled", 6, cx))
                            .on_change(cx.listener(|this, code: &str, _, cx| {
                                this.otp_typed = code.to_owned();
                                cx.notify();
                            }))
                            .into_any_element(),
                        para(
                            &if otp_typed.is_empty() {
                                "Nothing typed yet".to_owned()
                            } else {
                                format!("Value: {otp_typed}")
                            },
                            cx,
                        ),
                    ]),
                ),
                (
                    "On Complete",
                    col(vec![
                        h::InputOTP::new(self.demo_otp("otp-complete", 6, cx))
                            .on_complete(cx.listener(|this, code: &str, _, cx| {
                                this.otp_done = code.to_owned();
                                cx.notify();
                            }))
                            .into_any_element(),
                        para(
                            &if done.is_empty() {
                                "`onComplete` fires once every slot is filled".to_owned()
                            } else {
                                format!("Completed with {done}")
                            },
                            cx,
                        ),
                    ]),
                ),
                (
                    "Form Example",
                    col(vec![{
                        let state = self.demo_otp("otp-form", 6, cx);
                        h::Form::new()
                            .field(h::FormField::code("code", state.clone()).is_required(true))
                            .child(h::InputOTP::new(state).name("code"))
                            .child(h::Button::new("otp-form-submit").label("Verify"))
                            .into_any_element()
                    }]),
                ),
                (
                    "With Pattern",
                    col(vec![
                        spec(
                            "Digits (default)",
                            h::InputOTP::new(self.demo_otp("otp-pat-digits", 4, cx))
                                .pattern(h::OtpPattern::Digits),
                            cx,
                        ),
                        spec(
                            "Alphanumeric",
                            h::InputOTP::new(self.demo_otp("otp-pat-alnum", 4, cx))
                                .pattern(h::OtpPattern::Alphanumeric),
                            cx,
                        ),
                        spec(
                            "Any character",
                            h::InputOTP::new(self.demo_otp("otp-pat-any", 4, cx))
                                .pattern(h::OtpPattern::Any),
                            cx,
                        ),
                    ]),
                ),
                (
                    "With Validation",
                    col(vec![h::InputOTP::new(self.demo_otp("otp-validate", 6, cx))
                        .validate(|code| {
                            (code.chars().count() < 6)
                                .then(|| "Enter all six digits".into())
                        })
                        .into_any_element()]),
                ),
            ],""")

rep("""    pub fn page_input_otp(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let done = self.otp_done.clone();""",
    """    pub fn page_input_otp(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let done = self.otp_done.clone();
        let otp_typed = self.otp_typed.clone();""")

io.open(P, 'w', encoding='utf-8', newline='').write(s)
print('patched input otp page')
