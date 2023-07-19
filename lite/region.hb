<table>
    <tr>
        <th>Region</th>
        <th>Territories</th>
    </tr>
    {{#each region}}
    <tr>
        <td><a href="/region/Pacific Northwest">{{name}}</a></td>
        <td><table>
            <tr>
                <th>Territory</th>
                <th>Owner</th>
            </tr>
            {{#each territory}}
            <tr>
                <td><a href="/territory/{{name}}">{{name}}</a></td>
                <td><a href="/team/{{owner}}">{{owner}}</a></td>
            </tr>
            {{/each}}
        </table></td>
    </tr>
    {{/each}}
</table>