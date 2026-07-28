#ifndef _H_IRISNATIVEOBJECTEXPRESSION_
#define _H_IRISNATIVEOBJECTEXPRESSION_

#include <string>
using namespace std;

#include "IrisExpression.h"
#include "IrisComponents/IrisComponentsDefines.h"


class IrisNativeObjectExpression :
	public IrisExpression
{
protected:
	string m_strString = "";
	int m_nInteger = 0;
	double m_dFloat = 0.0;

	IrisNativeObjectType m_eType = IrisNativeObjectType::String;

public:

	virtual bool Generate();

	IrisNativeObjectExpression(IrisNativeObjectType eType, const string& strString);
	IrisNativeObjectExpression(int iInteger);
	IrisNativeObjectExpression(double dFloat);

	virtual ~IrisNativeObjectExpression();
};

#endif