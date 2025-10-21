use crate::{
    ScrapedEngineeringItem, ScrapedEngineeringItems,
    constant::ARMIN_RONACHER_ATOM_FEED_URL,
    xml::{XMLHandler, parse_xml_with},
};
use anyhow::{Context, Result, bail};
use quick_xml::reader::Reader;
use reqwest::StatusCode;

pub async fn request_lucumr_sitemap() -> Result<String> {
    let res = reqwest::get(ARMIN_RONACHER_ATOM_FEED_URL)
        .await
        .with_context(|| "Failed to request Armin Ronacher's Atom feed")?;
    if res.status() != StatusCode::OK {
        bail!(
            "Failed to request Armin Ronacher's Atom feed: {} - {}",
            res.status(),
            res.text().await?
        )
    } else {
        let xml = res.text().await?;
        Ok(xml)
    }
}

#[derive(Default)]
struct AtomFeed {
    items: ScrapedEngineeringItems,
    current_item: Option<ScrapedEngineeringItem>,
    current_element: String,
    current_text: String,
}

impl XMLHandler<ScrapedEngineeringItems> for AtomFeed {
    fn start(&mut self, name: &[u8]) -> Result<()> {
        match name {
            b"entry" => {
                self.current_item = Some(ScrapedEngineeringItem::default());
            }
            b"id" | b"title" | b"published" | b"updated" | b"content" => {
                self.current_element = String::from_utf8_lossy(name.as_ref()).to_string();
                self.current_text.clear();
            }
            _ => {}
        }
        Ok(())
    }

    fn text(&mut self, txt: &str) -> Result<()> {
        if !self.current_element.is_empty() {
            self.current_text.push_str(txt.trim());
        }
        Ok(())
    }

    fn end(&mut self, name: &[u8]) -> Result<()> {
        match name {
            b"entry" => {
                if let Some(entry) = self.current_item.take() {
                    self.items.push(entry);
                }
            }
            b"id" | b"title" | b"published" | b"updated" | b"content" => {
                if let Some(entry) = &mut self.current_item {
                    match self.current_element.as_str() {
                        "id" => entry.url = self.current_text.clone(),
                        "title" => entry.title = self.current_text.clone(),
                        "published" => {
                            if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&self.current_text)
                            {
                                entry.published = Some(dt.to_utc());
                            }
                        }
                        "updated" => {
                            if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&self.current_text)
                            {
                                entry.updated = Some(dt.to_utc());
                            }
                        }
                        "content" => entry.summary = Some(self.current_text.clone()),
                        _ => {}
                    }
                }
                self.current_element.clear();
                self.current_text.clear();
            }
            _ => {}
        }
        Ok(())
    }

    fn items(self) -> ScrapedEngineeringItems {
        self.items
    }
}

pub async fn scrape_lucumr_atom_feed() -> Result<ScrapedEngineeringItems> {
    let xml = request_lucumr_sitemap().await?;
    let reader = Reader::from_str(&xml);
    let handler = AtomFeed::default();
    let items = parse_xml_with(reader, handler)?;
    Ok(items)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_xml_entries() {
        let xml = r#"<entry>
            <id>https://lucumr.pocoo.org/2025/9/29/90-percent/</id>
            <title>90%</title>
            <link href="https://lucumr.pocoo.org/2025/9/29/90-percent/" />
            <published>2025-09-29T00:00:00+00:00</published>
            <updated>2025-09-29T00:00:00+00:00</updated>
            <author>
              <name>Armin Ronacher</name>
            </author>
            <content type="html"><![CDATA[<blockquote>
        <p>&#8220;I think we will be there in three to six months, where AI is writing 90% of
        the code. And then, in 12 months, we may be in a world where AI is writing
        essentially all of the code&#8221;</p>
        <p>â€” <a href="https://www.businessinsider.com/anthropic-ceo-ai-90-percent-code-3-to-6-months-2025-3">Dario Amodei</a></p>
        </blockquote>
        <p><a href="/2025/6/4/changes/">Three months ago</a> I said that AI changes everything.  I
        came to that after plenty of skepticism.  There are still good reasons to doubt
        that AI will write all code, but my current reality is close.</p>
        <p>For the infrastructure component I started at my new company, I&#8217;m probably
        north of 90% AI-written code.  I don&#8217;t want to convince you â€” just share what I
        learned.  In parts, because I approached this project differently from my first
        experiments with AI-assisted coding.</p>
        <p>The service is written in Go with few dependencies and an OpenAPI-compatible
        REST API.  At its core, it sends and receives emails.  I also generated SDKs
        for Python and TypeScript with a custom SDK generator.  In total: about 40,000
        lines, including Go, YAML, Pulumi, and some custom SDK glue.</p>
        <p>I set a high bar, especially that I can operate it reliably.  I&#8217;ve run similar
        systems before and knew what I wanted.</p>
        <h2>Setting it in Context</h2>
        <p>Some startups are already near 100% AI-generated.  I know, because many build
        in the open and you can see their code.  Whether that works long-term remains
        to be seen.  I still treat every line as my responsibility, judged as if I
        wrote it myself.  AI doesn&#8217;t change that.</p>
        <p>There are no weird files that shouldn&#8217;t belong there, no duplicate
        implementations, and no emojis all over the place.  The comments still follow
        the style I want and, crucially, often aren&#8217;t there.  I pay close attention to
        the fundamentals of system architecture, code layout, and database interaction.
        I&#8217;m incredibly opinionated.  As a result, there are certain things I don&#8217;t let
        the AI do.  I know it won&#8217;t reach the point where I could sign off on a commit.
        That&#8217;s why it&#8217;s not 100%.</p>
        <p>As contrast: another quick prototype we built is a mess of unclear database
        tgables, markdown file clutter in the repo, and boatloads of unwanted emojis.
        It served its purpose â€” validate an idea â€” but wasn&#8217;t built to last, and we had
        no expectation to that end.</p>
        <h2>Foundation Building</h2>
        <p>I began in the traditional way: system design, schema, architecture.  At this
        state I don&#8217;t let the AI write, but I loop it in AI as a kind of rubber duck.
        The back-and-forth helps me see mistakes, even if I don&#8217;t need or trust the
        answers.</p>
        <p>I did get the foundation wrong once.  I initially argued myself into a more
        complex setup than I wanted.  That&#8217;s a part where I later used the LLM to redo
        a larger part early and clean it up.</p>
        <p>For AI-generated or AI-supported code, I now end up with a stack that looks
        something like something I often wanted, but was too hard to do by hand:</p>
        <ul>
        <li>
        <p><strong>Raw SQL:</strong> This is probably the biggest change to how I used to write
        code.  I really like using an ORM, but I don&#8217;t like some of its effects.  In
        particular, once you approach the ORM&#8217;s limits, you&#8217;re forced to switch to
        handwritten SQL.  That mapping is often tedious because you lose some of the
        powers the ORM gives you.  Another consequence is that it&#8217;s very hard to find
        the underlying queries, which makes debugging harder.  Seeing the actual SQL
        in your code and in the database log is powerful.  You always lose that with
        an ORM.</p>
        <p>The fact that I no longer have to write SQL because the AI does it for me is
        a game changer.</p>
        <p>I also use raw SQL for migrations now.</p>
        </li>
        <li>
        <p><strong>OpenAPI first:</strong> I tried various approaches here.  There are many
        frameworks you can use.  I ended up first generating the OpenAPI specification
        and then using code generation from there to the interface layer.  This
        approach works better with AI-generated code.  The OpenAPI specification is
        now the canonical one that both clients and server shim is based on.</p>
        </li>
        </ul>
        <h2>Iteration</h2>
        <p>Today I use Claude Code and Codex. Each has strengths, but the constant is
        Codex for code review after PRs.  It&#8217;s very good at that.  Claude is
        indispensable still when debugging and needing a lot of tool access (eg: why do
        I have a deadlock, why is there corrupted data in the database etc.).  The
        working together of the two is where it&#8217;s most magical.  Claude might find the
        data, Codex might understand it better.</p>
        <p>I cannot stress enough how bad the code from these agents can be if you&#8217;re not
        careful.  While they understand system architecture and how to build something,
        they can&#8217;t keep the whole picture in scope.  They will recreate things that
        already exist.  They create abstractions that are completely inappropriate for
        the scale of the problem.</p>
        <p>You constantly need to learn how to bring the right information to the context.
        For me, this means pointing the AI to existing implementations and giving it
        very specific instructions on how to follow along.</p>
        <p>I generally create PR-sized chunks that I can review.  There are two paths to
        this:</p>
        <ol>
        <li>
        <p><strong>Agent loop with finishing touches:</strong> Prompt until the result is close,
        then clean up.</p>
        </li>
        <li>
        <p><strong>Lockstep loop:</strong> Earlier I went edit by edit. Now I lean on the first
        method most of the time, keeping a todo list for cleanups before merge.</p>
        </li>
        </ol>
        <p>It requires intuition to know when each approach is more likely to lead to the
        right results.  Familiarity with the agent also helps understanding when a task
        will not go anywhere, avoiding wasted cycles.</p>
        <h2>Where It Fails</h2>
        <p>The most important piece of working with an agent is the same as regular
        software engineering.  You need to understand your state machines, how the
        system behaves at any point in time, your database.</p>
        <p>It is easy to create systems that appear to behave correctly but have unclear
        runtime behavior when relying on agents.  For instance, the AI doesn&#8217;t fully
        comprehend threading or goroutines.  If you don&#8217;t keep the bad decisions at bay
        early it, you won&#8217;t be able to operate it in a stable manner later.</p>
        <p>Here&#8217;s an example: I asked it to build a rate limiter.  It &#8220;worked&#8221; but lacked
        jitter and used poor storage decisions.  Easy to fix if you know rate limiters,
        dangerous if you donâ€™t.</p>
        <p>Agents also operate on conventional wisdom from the internet and in tern do
        things I would never do myself.  It loves to use dependencies (particularly
        outdated ones).  It loves to swallow errors and take away all tracebacks.
        I&#8217;d rather uphold strong invariants and let code crash loudly when they fail,
        than hide problems.  If you don&#8217;t fight this, you end up with opaque,
        unobservable systems.</p>
        <h2>Where It Shines</h2>
        <p>For me, this has reached the point where I can&#8217;t imagine working any other way.
        Yes, I could probably have done it without AI.  But I would have built a
        different system in parts because I would have made different trade-offs.  This
        way of working unlocks paths I&#8217;d normally skip or defer.</p>
        <p>Here are some of the things I enjoyed a lot on this project:</p>
        <ul>
        <li>
        <p><strong>Research + code, instead of research and code later:</strong> Some things that
        would have taken me a day or two to figure out now take 10 to 15 minutes.<br />
        It allows me to directly play with one or two implementations of a problem.
        It moves me from abstract contemplation to hands on evaluation.</p>
        </li>
        <li>
        <p><strong>Trying out things:</strong> I tried three different OpenAPI implementations and
        approaches in a day.</p>
        </li>
        <li>
        <p><strong>Constant refactoring:</strong> The code looks more organized than it would
        otherwise have been because the cost of refactoring is quite low.  You need
        to know what you do, but if set up well, refactoring becomes easy.</p>
        </li>
        <li>
        <p><strong>Infrastructure:</strong> Claude got me through AWS and Pulumi.  Work I generally
        dislike became a few days instead of weeks.  It also debugged the setup issues
        as it was going through them.  I barely had to read the docs.</p>
        </li>
        <li>
        <p><strong>Adopting new patterns:</strong> While they suck at writing tests, they turned out
        great at setting up test infrastructure I didn&#8217;t know I needed.  I got a
        recommendation on Twitter to use
        <a href="https://golang.testcontainers.org/">testcontainers</a> for testing against
        Postgres.  The approach runs migrations once and then creates database clones
        per test.  That turns out to be super useful.  It would have been quite an
        involved project to migrate to.  Claude did it in an hour for all tests.</p>
        </li>
        <li>
        <p><strong>SQL quality:</strong> It writes solid SQL I could never remember.  I just need to
        review which I can.  But to this day I suck at remembering <code>MERGE</code> and <code>WITH</code>
        when writing it.</p>
        </li>
        </ul>
        <h2>What does it mean?</h2>
        <p>Is 90% of code going to be written by AI?  I don&#8217;t know.  What I do know is,
        that for me, on this project, the answer is already yes.  I&#8217;m part of that
        growing subset of developers who are building real systems this way.</p>
        <p>At the same time, for me, AI doesn&#8217;t own the code.  I still review every line,
        shape the architecture, and carry the responsibility for how it runs in
        production.  But the sheer volume of what I now let an agent generate would
        have been unthinkable even six months ago.</p>
        <p>That&#8217;s why I&#8217;m convinced this isn&#8217;t some far-off prediction.  It&#8217;s already here
        â€” just unevenly distributed â€” and the number of developers working like this is
        only going to grow.</p>
        <p>That said, none of this removes the need to actually be a good engineer.  If you
        let the AI take over without judgment, you&#8217;ll end up with brittle systems and
        painful surprises (data loss, security holes, unscalable software).  The tools
        are powerful, but they don&#8217;t absolve you of responsibility.</p>
        ]]></content>
          </entry>"#;
        let reader = Reader::from_str(xml);
        let handler = AtomFeed::default();
        let entries = parse_xml_with(reader, handler).expect("Failed to parse xml content");
        assert_eq!(entries.len(), 1);
        let first = entries.first().unwrap();
        assert!(first.title == "90%");
        assert!(first.url == "https://lucumr.pocoo.org/2025/9/29/90-percent/");
    }
}
