export const frontmatter = {"charmap":["Revan Juan","a","Stevano","b"],"title":"Dialog Plugin Demo"};
export default function PendonView() { return (<>
<h2>Dialog</h2>
<p>This is a naration. Excepteur anim <i>veniam</i> sunt occaecat non enim sunt sunt pariatur aliquip sint. Reprehenderit sunt ut ut duis eu do deserunt tempor consectetur minim cupidatat cupidatat ad.
</p>
<dl>
<dt class="a">Revan Juan</dt> <dd class="a"><q>Consectetur eu minim <i>aute</i> deserunt nulla amet elit.</q> <i>(esse qui cupidatat ex exercitation qui minim tempor anim mollit)</i></dd>
<dt class="b">Stevano</dt> <dd class="b"><i>(dolore eiusmod ad proident)</i> <q>Exercitation anim id culpa ex ea aliquip exercitation velit culpa. Aliquip occaecat ipsum fugiat eu nostrud velit sunt ex cillum.</q><br /><p>...</p><br /><q>Enim velit anim sunt qui mollit.</q></dd>
<dt class="a">Revan Juan</dt> <dd class="a"><q>In cillum cupidatat amet anim occaecat deserunt cupidatat.</q></dd>
</dl>

<dl>
<dt class="b">Stevano</dt> <dd class="b"><q>Nulla quis duis excepteur sint Lorem dolor anim adipisicing non aliqua consequat sunt irure esse.</q></dd>
</dl>

<p>Veniam duis adipisicing deserunt Lorem esse occaecat anim sit aliqua Lorem dolore aliquip. Mollit sit nisi culpa minim deserunt voluptate laboris irure veniam exercitation. Eu occaecat consectetur adipisicing deserunt amet cillum ullamco. Ea commodo in reprehenderit adipisicing laboris Lorem consequat cillum aute occaecat ut est sunt. Ullamco sint nulla tempor non non mollit tempor aute. Non occaecat velit et excepteur cupidatat velit sint sit eu consectetur reprehenderit laborum est sint.
</p>
<dl>
<dt>Rio</dt> <dd><q>Sunt exercitation veniam Lorem aliqua dolor esse officia.</q> <i>(eiusmod minim)</i><br /><q>Sint non elit consectetur exercitation.</q></dd>
</dl>

<h2>Expected Output</h2>
<pre lang="html"><code>&lt;p&gt;This is a naration. Excepteur anim &lt;em&gt;veniam&lt;/em&gt; sunt occaecat non enim sunt sunt pariatur aliquip sint. Reprehenderit sunt ut ut duis eu do deserunt tempor consectetur minim cupidatat cupidatat ad.&lt;/p&gt;

&lt;dl&gt;
&lt;dt class=&quot;a&quot;&gt;Revan Juan&lt;/dt&gt; &lt;dd class=&quot;a&quot;&gt;&lt;q&gt;Consectetur eu minim &lt;em&gt;aute&lt;/em&gt; deserunt nulla amet elit.&lt;/q&gt; &lt;i&gt;(esse qui cupidatat ex exercitation qui minim tempor anim mollit)&lt;/i&gt;&lt;/dd&gt;
&lt;dt class=&quot;b&quot;&gt;Stevano&lt;/dt&gt; &lt;dd class=&quot;b&quot;&gt;&lt;i&gt;(dolore eiusmod ad proident)&lt;/i&gt; &lt;q&gt;Exercitation anim id culpa ex ea aliquip exercitation velit culpa. Aliquip occaecat ipsum fugiat eu nostrud velit sunt ex cillum.&lt;/q&gt;&lt;br /&gt;&lt;p&gt;...&lt;/p&gt;&lt;br /&gt;&lt;q&gt;Enim velit anim sunt qui mollit.&lt;/q&gt;&lt;/dd&gt;
&lt;dt class=&quot;a&quot;&gt;Revan Juan&lt;/dt&gt; &lt;dd class=&quot;a&quot;&gt;&lt;q&gt;In cillum cupidatat amet anim occaecat deserunt cupidatat.&lt;/q&gt;&lt;/dd&gt;
&lt;/dl&gt;


&lt;dl&gt;
&lt;dt class=&quot;b&quot;&gt;Stevano&lt;/dt&gt; &lt;dd class=&quot;b&quot;&gt;&lt;q&gt;Nulla quis duis excepteur sint Lorem dolor anim adipisicing non aliqua consequat sunt irure esse..&lt;/q&gt;&lt;/dd&gt;
&lt;/dl&gt;

&lt;p&gt;Veniam duis adipisicing deserunt Lorem esse occaecat anim sit aliqua Lorem dolore aliquip. Mollit sit nisi culpa minim deserunt voluptate laboris irure veniam exercitation. Eu occaecat consectetur adipisicing deserunt amet cillum ullamco. Ea commodo in reprehenderit adipisicing laboris Lorem consequat cillum aute occaecat ut est sunt. Ullamco sint nulla tempor non non mollit tempor aute. Non occaecat velit et excepteur cupidatat velit sint sit eu consectetur reprehenderit laborum est sint.&lt;/p&gt;

&lt;dl&gt;
&lt;dt&gt;Rio&lt;/dt&gt; &lt;dd&gt;&lt;q&gt;Sunt exercitation veniam Lorem aliqua dolor esse officia.&lt;/q&gt; &lt;i&gt;(eiusmod minim)&lt;/i&gt;&lt;br /&gt;&lt;q&gt;Sint non elit consectetur exercitation.&lt;/q&gt;&lt;/dd&gt;
&lt;/dl&gt;
</code></pre>
<h2>Penjalasan</h2>
<ul>
<li>semua format markdown harus tetap bisa bekerja seperti biasa</li>
<li>plugin ini hanya override format dialognya saja yang selalu di awali dengan <code>foo: bar</code></li>
<li>di dalamnya, semua yang digapit <code>&quot;&quot;</code> dibungkus oleh <code>&lt;q&gt;</code> tanpa menambah kutip manual (kutip mengikuti default browser/CSS)</li>
<li>di dalamnya, semua yang digapit <code>_()_</code> atau <code>*()*</code> wajib dibungkus oleh <code>&lt;i&gt;</code>, bukan <code>&lt;em&gt;</code></li>
<li>di dalamnya, semua yang tidak digapit apapun dibungkus oleh <code>&lt;p&gt;</code></li>
<li>di dalamnya, <code>//</code> dirender menjadi <code>&lt;br /&gt;</code></li>
<li>1 line break terpisah = dl baru</li>
<li><code>charmap</code> didaptkan dari plugin pendon micromatter</li>
<li>karakter yang ada di dalam daftar charmap mendapatkan class spesial untuk dt dan ddnya.</li>
<li>jika tidak ada di charmap, maka tidak ada class khusus</li>
</ul>

</>); }
