//! The command module contains everything needed to perform strongly typed access
//! to commands associated with a message.

use std::ops::Range;
use std::slice::Iter;

/// An implementation of Iterator that iterates over the arguments of a `Message`.
#[derive(Clone)]
pub struct ArgumentIter<'a> {
    source: &'a str,
    iter: Iter<'a, Range<usize>>,
}

impl<'a> ArgumentIter<'a> {
    // This is intended for internal usage and thus hidden.
    #[doc(hidden)]
    pub fn new(source: &'a str, iter: Iter<'a, Range<usize>>) -> ArgumentIter<'a> {
        ArgumentIter {
            source: source,
            iter: iter
        }
    }
}

impl<'a> Iterator for ArgumentIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|range| &self.source[range.clone()])
    }
}

/// The `Command` trait is a trait that's implemented by types wishing to provide command
/// parsing capability for usage with the `Message::command` method.
pub trait Command<'a> {
    /// Provides the name of the command to be matched. Examples include `PRIVMSG` or `PING`.
    fn name() -> &'static str;

    /// This method takes in an iterator of arguments associated with a `Message` and attempts
    /// to parse the arguments into a matched `Command`.  If no match is found, None is returned.
    fn parse(arguments: ArgumentIter<'a>) -> Option<Self> where Self: Sized;

    /// A default implementation that takes in the given command name and arguments and attempts to match
    /// the command and parse the arguments into a strongly typed representation. If there is no match
    /// or the parse fails, it returns `None`.
    fn try_match(command: &str, arguments: ArgumentIter<'a>) -> Option<Self> where Self: Sized {
        if command == Self::name() {
            Self::parse(arguments)
        } else {
            None
        }
    }
}

/// Represents a PING command.  The first element is the host.
pub struct Ping<'a>(pub &'a str);

impl<'a> Command<'a> for Ping<'a> {
    fn name() -> &'static str {
        "PING"
    }

    fn parse(mut arguments: ArgumentIter<'a>) -> Option<Ping<'a>> {
        arguments.next().map(|suffix| Ping(suffix))
    }
}

/// Represents a PONG command. The first element is the host.
pub struct Pong<'a>(pub &'a str);

impl<'a> Command<'a> for Pong<'a> {
    fn name() -> &'static str {
        "PONG"
    }

    fn parse(mut arguments: ArgumentIter<'a>) -> Option<Pong<'a>> {
        arguments.next().map(|suffix| Pong(suffix))
    }
}

/// Represents a PONG command.  The first element is the target of the message and
/// the second eleement is the message.
pub struct Privmsg<'a>(pub &'a str, pub &'a str);

impl<'a> Command<'a> for Privmsg<'a> {
    fn name() -> &'static str {
        "PRIVMSG"
    }

    fn parse(mut arguments: ArgumentIter<'a>) -> Option<Privmsg<'a>> {
        arguments.next().and_then(|target| arguments.next().map(|suffix| Privmsg(target, suffix)))
    }
}

// Simple numerics are the numerics which have nothing but a single associated message. For
// these, we avail ourselves of a macro to define a suitable command implementation.
macro_rules! simple_numeric {
    // Hackyness to allow doc-comments; it looks kinda icky, but it works!
    ($(#[$meta:meta])* ($num:expr, $numeric_name:ident)) => (
        $(#[$meta])*
        pub struct $numeric_name<'a>(pub &'a str);

        impl<'a> Command<'a> for $numeric_name<'a> {
            fn name() -> &'static str {
                $num
            }

            fn parse(arguments: ArgumentIter<'a>) -> Option<$numeric_name<'a>> {
                arguments.skip(1).next().map(|x| $numeric_name(x))
            }
        }
    )
}

simple_numeric!{
  /// Represents a WELCOME numeric. The welcome message is the only element.
  ("001", Welcome)
}
simple_numeric!{
  /// Represents a YOURHOST numeric. The yourhost message is the only element.
  ("002", YourHost)
}
simple_numeric!{
  /// Represents a CREATED numeric. The created message is the only element.
  ("003", Created)
}
simple_numeric!{
  /// Represents a MYINFO numeric. The server info message is the only element.
  ("004", ServerInfo)
}

#[cfg(test)]
mod tests {
    use super::*;
    use message::*;

    #[test]
    fn test_ping_command() {
        let message = ping("test.host.com").unwrap();
        let Ping(host) = message.command::<Ping>().unwrap();

        assert_eq!("test.host.com", host);
    }

    #[test]
    fn test_pong_command() {
        let message = pong("test.host.com").unwrap();
        let Pong(host) = message.command::<Pong>().unwrap();

        assert_eq!("test.host.com", host);
    }

    #[test]
    fn test_privmsg_command() {
        let message = privmsg("#channel", "This is a message!").unwrap();
        let Privmsg(target, message) = message.command::<Privmsg>().unwrap();

        assert_eq!("#channel", target);
        assert_eq!("This is a message!", message);
    }

    #[test]
    fn test_welcome_numerc() {
        let msg = welcome("robots", "our overlords").unwrap();
        let Welcome(body) = msg.command::<Welcome>().unwrap();

        assert_eq!("our overlords", body);
    }
}
