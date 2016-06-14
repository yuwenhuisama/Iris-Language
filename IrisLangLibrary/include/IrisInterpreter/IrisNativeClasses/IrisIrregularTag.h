#ifndef _H_IRISIRREGULARTAG_
#define _H_IRISIRREGULARTAG_
#include <string>
using namespace std;

class IrisIrregularTag
{
private:
	size_t m_nLineNumber = 0;
	string m_strFileName = "";
	string m_strMessage = "";

public:
	IrisIrregularTag();
	void Initialize(size_t nLineNumber, const string& strFileName, const string& strMessage);
	~IrisIrregularTag();
};

#endif // !_H_IRISIRREGULARTAG_