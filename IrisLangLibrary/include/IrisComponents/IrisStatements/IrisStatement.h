#ifndef _H_IRISSTATEMENT
#define _H_IRISSTATEMENT
class IrisStatement
{
protected:
	int m_nLineNumber = 0;

public:
	IrisStatement();

	inline void SetLineNumber(unsigned int nLineNumber) { m_nLineNumber = nLineNumber; }
	inline unsigned int GetLineNumber() { return m_nLineNumber; }

	virtual bool Generate() = 0;
	virtual ~IrisStatement() = 0;
};

#endif